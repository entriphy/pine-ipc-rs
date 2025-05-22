use pine_ipc::{PINEBatch, PINECommand, PINE};

fn main() {
    // Connect to PINE using the default slot for PCSX2 (28011)
    let mut pine = PINE::connect("pcsx2", 28011, true).expect("Failed to connect to PCSX2");

    // Create batch command
    let mut batch = PINEBatch::new();
    batch.add(PINECommand::MsgTitle);
    batch.add(PINECommand::MsgGameVersion);
    batch.add(PINECommand::MsgRead32 { mem: 0x003667DC });

    // Send batch
    let res = pine.send(&mut batch).expect("Failed to send PINE batch");
    println!("{res:?}");
    // Example response:
    // [
    //     ResTitle { title: "Klonoa 2 - Lunatea's Veil" },
    //     ResGameVersion { version: "1.00" },
    //     ResRead32 { val: 3566512 }
    // ]
}
