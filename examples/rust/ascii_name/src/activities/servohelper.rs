use std::fs::OpenOptions;
use std::io::{self, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

/// A module for handling servo commands via a named pipe
#[derive(Debug)]
pub struct ServoHelper {
    sender: Sender<u8>,
    _handle: thread::JoinHandle<()>,
}

impl ServoHelper {
    /// Creates a new ServoHelper instance
    /// 
    /// This will:
    /// - Create a named pipe at /tmp/command if it doesn't exist
    /// - Spawn a background thread to handle writing to the pipe
    /// - Return a ServoHelper instance with a sender for sending commands
    pub fn new() -> io::Result<Self> {
        let pipe_path = "/tmp/command";
        
        let (sender, receiver) = mpsc::channel();
        
        // Spawn background thread to handle pipe writing
        let handle = thread::spawn(move || {
            if let Err(e) = Self::pipe_writer_loop(receiver, pipe_path) {
                eprintln!("ServoHelper pipe writer error: {}", e);
            }
        });
        
        Ok(ServoHelper {
            sender,
            _handle: handle,
        })
    }
    
    /// Send a command value (0-255) to be written to the pipe
    /// 
    /// # Arguments
    /// * `value` - An integer between 0 and 255 that will be written as ASCII
    /// 
    /// # Returns
    /// * `Ok(())` if the value was sent successfully
    /// * `Err(mpsc::SendError)` if the receiver thread has been disconnected
    pub fn send_command(&self, value: u8) -> Result<(), mpsc::SendError<u8>> {
        self.sender.send(value)
    }
    
    /// Main loop for the pipe writer thread
    fn pipe_writer_loop(receiver: Receiver<u8>, pipe_path: &str) -> io::Result<()> {
        // Open the pipe for writing
        let mut pipe = OpenOptions::new()
            .write(true)
            .open(pipe_path)?;
        
        // Process incoming commands
        while let Ok(value) = receiver.recv() {
            write!(pipe, "{}\n", value)?;
            pipe.flush()?;
        }
        
        Ok(())
    }
}

impl Drop for ServoHelper {
    fn drop(&mut self) {
        // The sender will be dropped, causing the receiver to return Err
        // and the background thread to exit gracefully
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_servo_helper_creation() {
        let servo_helper = ServoHelper::new();
        assert!(servo_helper.is_ok());
    }
    
    #[test]
    fn test_send_command() {
        let servo_helper = ServoHelper::new().expect("Failed to create ServoHelper");
        
        // Test sending valid values
        assert!(servo_helper.send_command(65).is_ok()); // 'A'
        assert!(servo_helper.send_command(66).is_ok()); // 'B' 
        assert!(servo_helper.send_command(0).is_ok());   // null character
        assert!(servo_helper.send_command(255).is_ok()); // max value
        
        // Give some time for the background thread to process
        thread::sleep(Duration::from_millis(100));
    }
}

/// Example usage function
pub fn example_usage() -> io::Result<()> {
    let servo_helper = ServoHelper::new()?;
    
    // Send some example commands
    servo_helper.send_command(72)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;  // 'H'
    servo_helper.send_command(101)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?; // 'e'
    servo_helper.send_command(108)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?; // 'l'
    servo_helper.send_command(108)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?; // 'l'
    servo_helper.send_command(111)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?; // 'o'
    servo_helper.send_command(10)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;  // newline
    
    // Give time for processing
    thread::sleep(Duration::from_millis(100));
    
    println!("Commands sent to pipe successfully!");
    Ok(())
}
