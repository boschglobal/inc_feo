# ServoHelper Module

This module provides a simple interface for sending ASCII commands through a named pipe located at `/tmp/command`.

## Features

- **Named Pipe Management**: Automatically creates a named pipe (FIFO) at `/tmp/command` if it doesn't exist
- **MPSC Queue**: Uses Rust's `std::sync::mpsc` channel for thread-safe communication
- **Background Processing**: Runs a background thread to handle writing to the pipe
- **Range Validation**: Accepts integer values between 0 and 255, writes them as ASCII characters
- **Shared Instance**: Can be shared across multiple activities using `Arc<ServoHelper>` to ensure only one pipe and background thread

## Usage

### Basic Example

```rust
use ascii_name::activities::servohelper::ServoHelper;
use std::io;

fn main() -> io::Result<()> {
    // Create the ServoHelper instance
    let servo_helper = ServoHelper::new()?;
    
    // Send ASCII values (0-255)
    servo_helper.send_command(65)?;  // 'A'
    servo_helper.send_command(66)?;  // 'B'
    servo_helper.send_command(10)?;  // newline
    
    Ok(())
}
```

### Shared Instance Example (for multiple activities)

```rust
use ascii_name::activities::components::create_shared_servo_helper;
use std::sync::Arc;
use std::io;

fn main() -> io::Result<()> {
    // Create a shared ServoHelper instance
    let servo_helper = create_shared_servo_helper()?;
    
    // Clone the Arc for use in multiple places
    let servo_helper1 = servo_helper.clone();
    let servo_helper2 = servo_helper.clone();
    
    // Both instances share the same pipe and background thread
    servo_helper1.send_command(65)?;  // 'A'
    servo_helper2.send_command(66)?;  // 'B'
    
    Ok(())
}
```

### Advanced Example

```rust
use ascii_name::activities::servohelper::ServoHelper;
use std::io;
use std::thread;
use std::time::Duration;

fn send_message(servo_helper: &ServoHelper, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    for byte in message.bytes() {
        servo_helper.send_command(byte)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        thread::sleep(Duration::from_millis(50)); // Optional delay
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let servo_helper = ServoHelper::new()?;
    
    // Send a message
    send_message(&servo_helper, "Hello World!\n")
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    // Send numeric characters
    for i in 0..10 {
        servo_helper.send_command(b'0' + i)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    }
    
    Ok(())
}
```

## API Reference

### `ServoHelper::new() -> io::Result<Self>`

Creates a new ServoHelper instance. This will:
- Create a named pipe at `/tmp/command` if it doesn't exist
- Spawn a background thread for handling pipe writes
- Return a ServoHelper instance ready to accept commands

### `ServoHelper::send_command(value: u8) -> Result<(), mpsc::SendError<u8>>`

Sends a command value (0-255) to be written to the pipe as an ASCII character.

**Parameters:**
- `value`: An integer between 0 and 255

**Returns:**
- `Ok(())` if the value was sent successfully
- `Err(mpsc::SendError<u8>)` if the receiver thread has been disconnected

## Testing the Pipe

You can test the pipe output in a separate terminal:

```bash
# In one terminal, run your program
cargo run --bin servo_example

# In another terminal, read from the pipe
cat /tmp/command
```

Or use `tail -f` to continuously monitor:

```bash
tail -f /tmp/command
```

## Implementation Details

- Uses `mkfifo` command to create the named pipe
- Employs a background thread to avoid blocking the main thread
- Automatically handles cleanup when the ServoHelper instance is dropped
- Thread-safe communication via MPSC channels

## Error Handling

The module handles several types of errors:
- Pipe creation failures
- File I/O errors when writing to the pipe
- Channel communication errors

Make sure to handle the returned `Result` types appropriately in your code.

## Notes

- The pipe will persist after the program exits
- Multiple readers can read from the same pipe
- Data is written immediately and flushed to ensure delivery
- The background thread will exit gracefully when the ServoHelper is dropped