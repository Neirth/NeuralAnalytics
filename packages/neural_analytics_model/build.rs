use std::process::Command;
use std::path::Path;
use std::fs;

fn main() {
    // Get the project root directory
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    // Path to the Python script
    let script_path = project_root.join("src/main.py");
    // Path to the build file in ./build
    let build_file_path = project_root.join("build/neural_analytics.onnx");

    // Check if the Python script exists
    if !script_path.exists() {
        panic!("[!] The file main.py is not found in the src folder");
    }

    // Check if the build file already exists
    if !build_file_path.exists() {
        // Call the Python script only if the build file does not exist
        let output = Command::new("python3")
            .arg(&script_path)
            .output()
            .expect("[!] Error executing the Python script");

        // Display output or errors from the script in the console
        if !output.status.success() {
            eprintln!(
                "[!] Error in the Python script: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        } else {
            println!(
                "[*] Script executed successfully: {}",
                String::from_utf8_lossy(&output.stdout)
            );
        }
    } else {
        println!("[*] The build file already exists, skipping script execution.");
    }

    // Copy the file neural_analytics.onnx
    let mut target_dir = Path::new("target")
        .join(std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string()))
        .join("assets");

    target_dir = project_root.join("../../").join(target_dir);

    // Create the target directory if it does not exist
    fs::create_dir_all(&target_dir).expect("[!] OS: Error creating the target directory");

    // Copy the file
    let target_path = target_dir.join("neural_analytics.onnx");
    fs::copy(&build_file_path, &target_path).expect("[!] OS: Error copying the file");

    println!("[*] File copied to: {:?}", target_path);
}