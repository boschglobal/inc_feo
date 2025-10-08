use ascii_name::activities::servohelper::ServoHelper;
use std::io;
use std::thread;
use std::time::Duration;

fn main() -> io::Result<()> {
    println!("ServoHelper Example - Writing to /tmp/command pipe");
    
    // Create the ServoHelper instance
    let servo_helper = ServoHelper::new()?;
    
    
    // Send some numeric values
    println!("Sending numeric values 0-255...");
    for i in (0..=255).step_by(5) {
        servo_helper.send_command(i)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        thread::sleep(Duration::from_millis(100));
    }
    
    
    println!("Example completed! Check the pipe output.");
    
    // Keep the program running for a bit to ensure all data is written
    thread::sleep(Duration::from_millis(500));
    
    Ok(())
}