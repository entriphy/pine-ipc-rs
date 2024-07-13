# pine-rs
Dependencyless Rust implementation of the [PINE IPC protocol](https://github.com/GovanifY/pine).

## Usage
```rust
use pine_ipc::{PINEBatch, PINECommand, PINEResult, PINE};

fn main() {
    // Connect to PINE using the default slot for PCSX2 (28011)
    let mut pine = PINE::connect_pcsx2(None).unwrap();
    // alternatively: PINE::connect("pcsx2", 28011, false);

    // Create batch command
    let mut batch = PINEBatch::new();
    batch.add(PINECommand::MsgTitle);
    batch.add(PINECommand::MsgGameVersion);
    batch.add(PINECommand::MsgRead32 { mem: 0x003667DC });

    // Send batch command
    let res = pine.send(&mut batch).unwrap();
    match res {
        // Output example:
        // [
        //     ResTitle { title: "Klonoa 2 - Lunatea's Veil" }, 
        //     ResGameVersion { version: "1.00" },
        //     ResRead32 { val: 3566512 }
        // ]
        PINEResult::Ok(ans) => println!("{ans:?}"),
        PINEResult::Fail => println!("Command failed"),
    }

    // Shutdown
    pine.shutdown().unwrap();
}
```