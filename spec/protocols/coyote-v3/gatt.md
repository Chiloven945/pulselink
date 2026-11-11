# Coyote V3 GATT

| Service  | Characteristic | Properties  | Purpose   |
|----------|----------------|-------------|-----------|
| `0x180C` | `0x150A`       | Write       | Commands  |
| `0x180C` | `0x150B`       | Notify      | Responses |
| `0x180A` | `0x1500`       | Read/Notify | Battery   |

Base UUID: `0000xxxx-0000-1000-8000-00805f9b34fb`.

Stimulator 3.0 device name: `47L121000`.

The SDK must check characteristic properties and choose a supported write mode instead of assuming
`WriteWithoutResponse`.
