use std::process::Command;
use std::path::Path;
use std::fs;
use std::io;

fn install_requirements(project_root: &Path) {
    let requirements_path = project_root.join("requirements.txt");
    
    if !requirements_path.exists() {
        panic!("[!] Requirements.txt file not found in project directory");
    }

    println!("[*] Installing Python dependencies...");
    let output = Command::new("pip3")
        .args(["install", "-r"])
        .arg(&requirements_path)
        .output()
        .expect("[!] Error executing pip");

    if !output.status.success() {
        panic!(
            "[!] Error installing dependencies: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    println!("[*] Dependencies installed successfully");
}

fn build_model(script_path: &Path, build_file_path: &Path) -> io::Result<()> {
    if !script_path.exists() {
        panic!("[!] File main.py not found in src folder");
    }

    if !build_file_path.exists() {
        let output = Command::new("python3")
            .current_dir(script_path.parent().unwrap())
            .arg(script_path)
            .output()?;

        if !output.status.success() {
            eprintln!(
                "[!] Python script error: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(io::Error::new(io::ErrorKind::Other, "Python script failed"));
        }

        println!(
            "[*] Script executed successfully: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    } else {
        println!("[*] Build file already exists, skipping script execution");
    }

    Ok(())
}

fn copy_model_assets(build_file_path: &Path, project_root: &Path) -> io::Result<()> {
    let mut target_dir = Path::new("target")
        .join(std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string()))
        .join("assets");

    target_dir = project_root.join("../../").join(target_dir);

    fs::create_dir_all(&target_dir)
        .map_err(|e| io::Error::new(e.kind(), "[!] Error creating target directory"))?;

    let target_path = target_dir.join("neural_analytics.onnx");
    fs::copy(build_file_path, &target_path)
        .map_err(|e| io::Error::new(e.kind(), "[!] Error copying file"))?;

    println!("[*] File copied to: {:?}", target_path);
    Ok(())
}

fn main() {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    
    install_requirements(&project_root);

    let script_path = project_root.join("src/main.py");
    let build_file_path = project_root.join("build/neural_analytics.onnx");

    build_model(&script_path, &build_file_path)
        .expect("[!] Failed to build model");
        
    copy_model_assets(&build_file_path, project_root)
        .expect("[!] Failed to copy model assets");
}