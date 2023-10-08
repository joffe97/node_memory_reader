# Node memory reader
This is a package for node that reads data from memory from a given process.

## Methods
### readMemory
```ts
readMemory(pid: number, address: number, length: number) -> number[]
```

##### Arguments
* pid: Process id of the process that are being read
* address: Relative memory address of the first byte
* length: Number of bytes

##### Return
The returned value is an array of `length` amount of bytes; starting from the relative `address` in the process with process id `pid`.

##### Example
```ts
import * as nodeMemoryReader from "node_memory_reader";
let memory: number[] = nodeMemoryReader.readMemory(41462, 0x4052a0, 20);
```


### readMemoryWithDataSize
```ts
readMemoryWithDataSize(pid: number, address: number, length: number, dataTypeSize: DataTypeSize) -> number[]
```

##### Arguments
* pid: Process id of the process that are being read
* address: Relative memory address of the first byte
* length: Number of bytes
* dataTypeSize: Size of the returned data size

##### Return
The returned value is an array of `length` amount of values with the size of `dataTypeSize`; starting from the relative `address` in the process with process id `pid`.

##### Example
```ts
import * as nodeMemoryReader from "node_memory_reader";
let memory: number[] = nodeMemoryReader.readMemoryWithDataSize(41462, 0x4052a0, 1, nodeMemoryReader.B32);
```


## Note
This package is compatible with Linux and Windows.

MacOS support is not considered at the moment.
