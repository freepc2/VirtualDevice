# Board memory architecture

The crate models one board's input and output memory images. `BoardType` selects the board connection type and digital representation. `IoAddress` stores the direction, physical address, offset, access kind, and a precomputed absolute byte index.

`BoardMemory` owns the input and output byte buffers. Direction-specific digital methods operate on the selected buffer; analog methods encode and decode little-endian values, and byte methods copy raw ranges. Buffer start addresses translate the absolute address index to an image-relative offset.

Supported board types:

- TwinCAT2: bit-packed digital values.
- TwinCAT3: one byte per digital value; bit position contributes to the precomputed byte index.
- Comizoa: bit-packed digital values with a board ID connection.

The crate has no server, network registry, or emulator layer. Applications can build those around `BoardMemory` as needed.
