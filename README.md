# Node memory reader
This is a package for node that reads bytes from memory for a given process.

## Methods
```ts
read_memory(pid: number, address: number, length: number) -> number[]
```

##### Arguments
* pid: Process id of the process that are being read
* address: Relative memory address of the first byte
* length: Number of bytes

##### Return
The returned value is an array of `length` amount of bytes; starting from the relative `address` in the process with process id `pid`.

#### Examples
##### Javascript
```js
const node_memory_reader = require("node_memory_reader")
let memory = node_memory_reader.read_memory(41462, 0x4052a0, 20);
```

##### Typescript
```ts
import * as node_memory_reader from "node_memory_reader";
let memory: number[] = node_memory_reader.read_memory(41462, 0x4052a0, 20);
```
