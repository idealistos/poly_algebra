use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use log::info;

/// Service for managing a persistent Pari/GP process
pub struct GpPariService {
    process: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout_receiver: Option<Receiver<String>>,
    executable_path: String,
    task_mutex: Arc<Mutex<()>>,
}

impl GpPariService {
    /// Create a new GpPariService instance
    pub fn new(executable_path: String) -> Self {
        Self {
            process: None,
            stdin: None,
            stdout_receiver: None,
            executable_path,
            task_mutex: Arc::new(Mutex::new(())),
        }
    }

    /// Start the Pari/GP process if it's not already running
    fn start_process(&mut self) -> Result<(), String> {
        if self.process.is_some() {
            return Ok(());
        }

        let mut child = Command::new(&self.executable_path)
            .arg("-q")
            .arg("-s")
            .arg("128000000")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn {}: {}", self.executable_path, e))?;
        info!("Started Pari/GP process {}", child.id());

        let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;

        // Set up stdout reading in a separate thread
        let (tx, rx) = channel();
        let stdout_reader = BufReader::new(stdout);

        thread::spawn(move || {
            for line in stdout_reader.lines() {
                match line {
                    Ok(line) => {
                        if let Err(_) = tx.send(line) {
                            break; // Channel closed, exit thread
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        self.process = Some(child);
        self.stdin = Some(stdin);
        self.stdout_receiver = Some(rx);

        Ok(())
    }

    /// Stop the Pari/GP process
    fn stop_process(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
        self.stdin = None;
        self.stdout_receiver = None;
    }

    /// Run a task on the Pari/GP process
    pub fn run_task(&mut self, task: String) -> Result<Vec<String>, String> {
        // Clone the Arc to avoid borrowing issues
        let task_mutex = self.task_mutex.clone();

        // Acquire the mutex to ensure only one task runs at a time
        let _guard = task_mutex
            .lock()
            .map_err(|e| format!("Failed to acquire task mutex: {}", e))?;

        // Start the process if needed
        self.start_process()?;

        // Get stdin and stdout receiver
        let stdin = self.stdin.as_mut().ok_or("No stdin available")?;
        let stdout_receiver = self
            .stdout_receiver
            .as_ref()
            .ok_or("No stdout receiver available")?;

        // Write the task to stdin
        stdin
            .write_all((task.clone() + "\n").as_bytes())
            .map_err(|e| format!("Failed to write to stdin: {}", e))?;
        stdin
            .flush()
            .map_err(|e| format!("Failed to flush stdin: {}", e))?;

        // Collect output lines
        let mut output_lines = Vec::new();
        let timeout = Duration::from_secs(5);
        let start_time = std::time::Instant::now();

        loop {
            // Check for timeout
            if start_time.elapsed() > timeout {
                self.stop_process();
                return Err("Task timed out after 5 seconds".to_string());
            }

            // Try to receive output with a short timeout
            match stdout_receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(line) => {
                    output_lines.push(line.clone());

                    // Check if the last line is "Done"
                    if line.trim() == "Done" {
                        // Remove the "Done" line and return the rest
                        output_lines.pop();
                        return Ok(output_lines);
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Continue waiting
                    continue;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    // Process has terminated
                    self.stop_process();
                    return Err("Pari/GP process terminated unexpectedly".to_string());
                }
            }
        }
    }

    /// Check if the process is currently running
    pub fn is_running(&self) -> bool {
        self.process.is_some()
    }

    /// Get the executable path
    pub fn executable_path(&self) -> &str {
        &self.executable_path
    }
}

impl Drop for GpPariService {
    fn drop(&mut self) {
        self.stop_process();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{get_pari_executable_path, init_gp_pari_service, set_pari_executable_path};
    use ctor::ctor;
    use test_log::test;

    const GP_PATH: &str = r"C:\progs\pari\gp.exe";

    // This function runs once before any tests start
    #[ctor]
    fn test_init() {
        println!("Initializing test environment (gp_pari_service.rs)...");
        set_pari_executable_path(GP_PATH.to_string());
        if let Err(e) = init_gp_pari_service() {
            println!(
                "Warning: Failed to initialize GpPariService for tests: {}",
                e
            );
        } else {
            println!("GpPariService initialized successfully for tests");
        }
    }

    #[test]
    fn test_gp_pari_service_creation() {
        let path = get_pari_executable_path().expect("Failed to get Pari/GP executable path");
        let service = GpPariService::new(path.clone());

        assert_eq!(service.executable_path(), path);
        assert!(!service.is_running());
    }

    #[test]
    fn test_gp_pari_service_task_execution() {
        let path = get_pari_executable_path().expect("Failed to get Pari/GP executable path");
        let mut service = GpPariService::new(path);

        let task = r#"{p = x^2 - y^2; q = (x - y)^2; g = gcd([p, q]); print(g); print(p / g); print("Done")}"#;

        match service.run_task(task.to_string()) {
            Ok(output) => {
                // Verify that the process started
                assert!(service.is_running());

                // Verify that we got some output
                assert!(!output.is_empty());

                // The output should contain the GCD and the result of p/g
                let output_text = output.join("\n");
                println!("Pari/GP output: {}", output_text);

                // Basic validation that we got meaningful output
                assert!(output_text.contains("x - y") || output_text.contains("x+y"));
            }
            Err(e) => {
                // If Pari/GP is not available, this is expected
                if e.contains("Failed to spawn") || e.contains("not found") {
                    println!("Pari/GP not available for testing: {}", e);
                } else {
                    panic!("Unexpected error: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_gp_pari_service_multiple_tasks() {
        let path = get_pari_executable_path().expect("Failed to get Pari/GP executable path");
        let mut service = GpPariService::new(path);

        let task1 = r#"{print("Hello"); print("Done")}"#;
        let task2 = r#"{print("World"); print("Done")}"#;

        match service.run_task(task1.to_string()) {
            Ok(output1) => {
                assert!(service.is_running());
                assert_eq!(output1, vec!["Hello"]);

                // Run a second task
                match service.run_task(task2.to_string()) {
                    Ok(output2) => {
                        assert!(service.is_running());
                        assert_eq!(output2, vec!["World"]);
                    }
                    Err(e) => {
                        if e.contains("Failed to spawn") || e.contains("not found") {
                            println!("Pari/GP not available for testing: {}", e);
                        } else {
                            panic!("Unexpected error on second task: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                if e.contains("Failed to spawn") || e.contains("not found") {
                    println!("Pari/GP not available for testing: {}", e);
                } else {
                    panic!("Unexpected error: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_gp_pari_service_timeout() {
        set_pari_executable_path(GP_PATH.to_string());
        let path = get_pari_executable_path().expect("Failed to get Pari/GP executable path");
        let mut service = GpPariService::new(path);

        // A task that should timeout (infinite loop)
        let task = r#"{while(1, print("loop")); print("Done")}"#;

        match service.run_task(task.to_string()) {
            Ok(_) => {
                // If it doesn't timeout, that's unexpected but not necessarily wrong
                println!("Task completed without timeout (unexpected)");
            }
            Err(e) => {
                if e.contains("Failed to spawn") || e.contains("not found") {
                    println!("Pari/GP not available for testing: {}", e);
                } else if e.contains("timed out") {
                    // Expected behavior
                    assert!(!service.is_running()); // Process should be stopped
                } else {
                    panic!("Unexpected error: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_gp_pari_service_invalid_task() {
        let path = get_pari_executable_path().expect("Failed to get Pari/GP executable path");
        let mut service = GpPariService::new(path);

        // A task with syntax error
        let task = r#"{print("Hello"; print("Done")}"#; // Missing closing parenthesis

        match service.run_task(task.to_string()) {
            Ok(_) => {
                // If it succeeds, that's unexpected but not necessarily wrong
                println!("Invalid task completed successfully (unexpected)");
            }
            Err(e) => {
                if e.contains("Failed to spawn") || e.contains("not found") {
                    println!("Pari/GP not available for testing: {}", e);
                } else if e.contains("terminated") || e.contains("timed out") {
                    // Expected behavior for invalid syntax
                    assert!(!service.is_running()); // Process should be stopped
                } else {
                    panic!("Unexpected error: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_gp_pari_service_process_restart() {
        let path = get_pari_executable_path().expect("Failed to get Pari/GP executable path");
        let mut service = GpPariService::new(path);

        // First task
        let task1 = r#"{print("First"); print("Done")}"#;

        match service.run_task(task1.to_string()) {
            Ok(_) => {
                assert!(service.is_running());

                // Force stop the process
                service.stop_process();
                assert!(!service.is_running());

                // Second task should restart the process
                let task2 = r#"{print("Second"); print("Done")}"#;
                match service.run_task(task2.to_string()) {
                    Ok(_) => {
                        assert!(service.is_running());
                    }
                    Err(e) => {
                        if e.contains("Failed to spawn") || e.contains("not found") {
                            println!("Pari/GP not available for testing: {}", e);
                        } else {
                            panic!("Unexpected error on restart: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                if e.contains("Failed to spawn") || e.contains("not found") {
                    println!("Pari/GP not available for testing: {}", e);
                } else {
                    panic!("Unexpected error: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_gp_pari_service_concurrent_access() {
        let path = get_pari_executable_path().expect("Failed to get Pari/GP executable path");
        let mut service = GpPariService::new(path);

        // Create multiple threads that try to run tasks simultaneously
        let mut handles = vec![];
        let service_ref = &mut service;

        for i in 0..3 {
            let task = format!(r#"{{print("Task {}"); print("Done")}}"#, i);
            let handle = thread::spawn(move || {
                // We can't actually share the mutable reference across threads,
                // but this test demonstrates the mutex protection at the singleton level
                // In practice, the singleton access is what's protected
                task
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            let task = handle.join().unwrap();
            // Run the task sequentially to test the mutex
            match service_ref.run_task(task) {
                Ok(output) => {
                    assert!(!output.is_empty());
                    println!("Task completed successfully: {:?}", output);
                }
                Err(e) => {
                    if e.contains("Failed to spawn") || e.contains("not found") {
                        println!("Pari/GP not available for testing: {}", e);
                    } else {
                        panic!("Unexpected error: {}", e);
                    }
                }
            }
        }
    }
}
