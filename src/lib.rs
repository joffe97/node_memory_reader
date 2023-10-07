use neon::prelude::*;
use read_process_memory::{self, Pid, ProcessHandle};
use std::{convert::TryInto, io};

fn pid_to_handle(pid: Pid) -> io::Result<ProcessHandle> {
    Ok(pid.try_into()?)
}

fn vec_to_jsarray<'a, C: Context<'a>>(cx: &mut C, vector: Vec<u8>) -> JsResult<'a, JsArray> {
    let buffer = JsArray::new(cx, vector.len() as u32);
    for (i, s) in vector.into_iter().enumerate() {
        let v = cx.number(s);
        buffer.set(cx, i as u32, v)?;
    }
    Ok(buffer)
}

fn read_memory_inner(address: usize, length: usize, pid: usize) -> Result<Vec<u8>, std::io::Error> {
    let pid_object = pid.try_into().or(Err(std::io::Error::new(
        io::ErrorKind::NotFound,
        "pid not found",
    )))?;
    let handle = pid_to_handle(pid_object)?;
    read_process_memory::copy_address(address, length, &handle)
}

fn read_memory(mut cx: FunctionContext) -> JsResult<JsArray> {
    let pid_f64 = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let address_f64 = cx.argument::<JsNumber>(1)?.value(&mut cx);
    let length_f64 = cx.argument::<JsNumber>(2)?.value(&mut cx);

    let address: usize = address_f64.floor() as usize;
    let length: usize = length_f64.floor() as usize;
    let pid: usize = pid_f64.floor() as usize;

    let memory = match read_memory_inner(address, length, pid) {
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
    cx.export_function("read_memory", read_memory)?;
    Ok(())
}
