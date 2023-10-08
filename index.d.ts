export enum DataTypeSize {
    B8 = 1,
    B16 = 2,
    B32 = 4,
}

export declare function readMemory(pid: number, address: number, length: number): number[];
export declare function readMemoryWithDataSize(pid: number, address: number, length: number, dataTypeSize: DataTypeSize): number[];