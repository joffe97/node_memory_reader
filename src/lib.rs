use anyhow::{anyhow, Result};
use neon::prelude::*;
use read_process_memory::{self, Pid, ProcessHandle};
use std::{convert::TryInto, io};

const BITS_IN_BYTE: usize = 8;

enum DataTypeSize {
    B8,
    B16,
    B32,
}

impl DataTypeSize {
    fn byte_size(&self) -> u32 {
        match self {
            Self::B8 => 1,
            Self::B16 => 2,
            Self::B32 => 4,
        }
    }
}

fn pid_to_handle(pid: Pid) -> io::Result<ProcessHandle> {
    Ok(pid.try_into()?)
}

fn vec_to_jsarray<'a, T, C>(cx: &mut C, vector: Vec<T>) -> JsResult<'a, JsArray>
where
    f64: From<T>,
    C: Context<'a>,
{
    let buffer = JsArray::new(cx, vector.len() as u32);
    for (i, s) in vector.into_iter().enumerate() {
        let v = cx.number(s);
        buffer.set(cx, i as u32, v)?;
    }
    Ok(buffer)
}

fn bytes_to_num(vector: &[u8], index: usize, length: usize) -> Option<u32> {
    let bytes = (index..index + length)
        .map(|i| vector.get(i))
        .collect::<Option<Vec<&u8>>>()?;

    let num = bytes.into_iter().enumerate().fold(0, |acc, (i, byte)| {
        let shift_size = i * BITS_IN_BYTE;
        let shifted_byte = (u32::from(*byte)) << shift_size;
        return acc + shifted_byte;
    });
    Some(num)
}

fn byte_vec_to_value_vec(vector: &[u8], bytes_per_value: usize) -> Result<Vec<u32>> {
    (0..vector.len())
        .step_by(bytes_per_value)
        .map(|i| match bytes_to_num(vector, i, bytes_per_value) {
            Some(number) => Ok(number),
            None => Err(anyhow!("")),
        })
        .collect()
}

fn read_memory(pid: usize, address: usize, length: usize) -> Result<Vec<u8>> {
    let pid_object = pid.try_into().or(Err(std::io::Error::new(
        io::ErrorKind::NotFound,
        "pid not found",
    )))?;
    let handle = pid_to_handle(pid_object)?;
    read_process_memory::copy_address(address, length, &handle).map_err(|err| anyhow!(err))
}

fn read_memory_with_data_size(
    pid: usize,
    address: usize,
    length: usize,
    data_size: usize,
) -> Result<Vec<u32>> {
    let total_data_size = length * data_size;
    let memory = read_memory(pid, address, total_data_size)?;
    byte_vec_to_value_vec(&memory, data_size)
}

fn data_type_size_js_creator<'a, C: Context<'a>>(cx: &mut C) -> JsResult<'a, JsObject> {
    let data_type_size_js = cx.empty_object();
    let b8_js = cx.number(DataTypeSize::B8.byte_size());
    let b16_js = cx.number(DataTypeSize::B16.byte_size());
    let b32_js = cx.number(DataTypeSize::B32.byte_size());
    data_type_size_js.set(cx, "B8", b8_js)?;
    data_type_size_js.set(cx, "B16", b16_js)?;
    data_type_size_js.set(cx, "B32", b32_js)?;
    Ok(data_type_size_js)
}

fn read_memory_js_wrapper(mut cx: FunctionContext) -> JsResult<JsArray> {
    let pid = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize;
    let address = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let length = cx.argument::<JsNumber>(2)?.value(&mut cx) as usize;

    let memory = match read_memory(pid, address, length) {
        Ok(memory) => memory,
        Err(error) => {
            return cx.throw_error(format!(
                "cannot read from given memory: {}",
                error.to_string()
            ));
        }
    };

    let jsarray = vec_to_jsarray(&mut cx, memory)?;
    Ok(jsarray)
}

fn read_memory_with_data_size_js_wrapper(mut cx: FunctionContext) -> JsResult<JsArray> {
    let pid = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize;
    let address = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let length = cx.argument::<JsNumber>(2)?.value(&mut cx) as usize;
    let data_size = cx.argument::<JsNumber>(3)?.value(&mut cx) as usize;

    let memory = match read_memory_with_data_size(pid, address, length, data_size) {
        Ok(memory) => memory,
        Err(error) => {
            return cx.throw_error(format!(
                "cannot read from given memory: {}",
                error.to_string()
            ));
        }
    };

    let jsarray = vec_to_jsarray(&mut cx, memory)?;
    Ok(jsarray)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("readMemory", read_memory_js_wrapper)?;
    cx.export_function(
        "readMemoryWithDataSize",
        read_memory_with_data_size_js_wrapper,
    )?;
    let data_type_size_js = data_type_size_js_creator(&mut cx)?;
    cx.export_value("DataTypeSize", data_type_size_js)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{read_memory, read_memory_with_data_size, BITS_IN_BYTE};

    #[test]
    fn test_read_memory_inner_no_error() {
        let pid = std::process::id();
        let pid_ref = &pid;
        let length = 4;
        let address = pid_ref as *const u32;
        read_memory(pid as usize, address as usize, length as usize).unwrap();
    }

    #[test]
    fn test_read_memory_with_data_size_8b_correct_return_count() {
        let variable: u8 = 0b10010110;
        let variable_ref = &variable;

        let pid = std::process::id();
        let length = 1 as u8;
        let address = variable_ref as *const u8;

        let memory =
            read_memory_with_data_size(pid as usize, address as usize, length as usize, 1).unwrap();
        assert_eq!(memory.len(), length as usize);
    }

    #[test]
    fn test_read_memory_with_data_size_16b_correct_return_count() {
        let variable: u16 = 0b10010110_01011010;
        let variable_ref = &variable;

        let pid = std::process::id();
        let length = 1 as u16;
        let address = variable_ref as *const u16;

        let memory =
            read_memory_with_data_size(pid as usize, address as usize, length as usize, 2).unwrap();
        assert_eq!(memory.len(), length as usize);
    }

    #[test]
    fn test_read_memory_with_data_size_32b_correct_return_count() {
        let variable = 0b1001 as u32;
        let variable_ref = &variable;

        let pid = std::process::id();
        let length = 1 as u32;
        let address = variable_ref as *const u32;

        let memory =
            read_memory_with_data_size(pid as usize, address as usize, length as usize, 4).unwrap();
        assert_eq!(memory.len(), length as usize);
    }

    #[test]
    fn test_read_memory_with_data_size_2x16b_correct_return_count() {
        let variable: u32 = 0b10010110_01011010_11001100_00110011;
        let variable_ref = &variable;

        let pid = std::process::id();
        let length = 2 as u32;
        let address = variable_ref as *const u32;

        let memory =
            read_memory_with_data_size(pid as usize, address as usize, length as usize, 2).unwrap();
        assert_eq!(memory.len(), length as usize);
    }

    #[test]
    fn test_read_memory_with_data_size_2x16b_correct_value() {
        let variable: u32 = 0b10010110_01011010_11001100_00110011;
        let variable_ref = &variable;

        let pid = std::process::id();
        let length = 2 as u32;
        let address = variable_ref as *const u32;

        let memory =
            read_memory_with_data_size(pid as usize, address as usize, length as usize, 2).unwrap();
        assert_eq!(memory[0], variable & 0b00000000_00000000_11111111_11111111);
        assert_eq!(memory[1], (variable >> length * BITS_IN_BYTE as u32));
    }

    #[test]
    fn test_read_memory_with_data_size_8b_correct_value() {
        let variable: u8 = 0b10010110;
        let variable_ref = &variable;

        let pid = std::process::id();
        let length = 1 as u32;
        let address = variable_ref as *const u8;

        let memory =
            read_memory_with_data_size(pid as usize, address as usize, length as usize, 1).unwrap();
        assert_eq!(memory[0] as u8, variable);
    }

    #[test]
    fn test_read_memory_with_data_size_16b_correct_value() {
        let variable: u16 = 0b10010110_01011010;
        let variable_ref = &variable;

        let pid = std::process::id();
        let length = 1 as u16;
        let address = variable_ref as *const u16;

        let memory =
            read_memory_with_data_size(pid as usize, address as usize, length as usize, 2).unwrap();
        assert_eq!(memory[0] as u16, variable);
    }

    #[test]
    fn test_read_memory_with_data_size_32b_correct_value() {
        let variable = 0b10010110_01011010_11001100_00110011;
        let variable_ref = &variable;

        let pid = std::process::id();
        let length = 1 as u32;
        let address = variable_ref as *const u32;

        let memory =
            read_memory_with_data_size(pid as usize, address as usize, length as usize, 4).unwrap();
        assert_eq!(memory[0], variable);
    }
}
