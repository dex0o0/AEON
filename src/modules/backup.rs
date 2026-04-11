use std::fs::{File, create_dir_all};
use std::path::PathBuf;
use std::path::Path;
use tar::Builder;
use flate2::write::GzEncoder;
use chrono::Local;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct BackupConfig {
    pub source_path: PathBuf,
    pub output_dir: PathBuf,
    pub compression: CompressionType,
    pub compression_level: u8,
    pub exclude_patterns: Vec<String>,
}
#[derive(Debug)]
pub enum CompressionType {
    Gzip,
    Bzip2,
    Xz,
    None,
}

pub fn create_backup(config: BackupConfig) -> Result<(), String> {
    // Step 1: Validate paths
    if !config.source_path.exists() {
        return Err(format!("Source path does not exist: {}", config.source_path.display()));
    }
    
    // Step 2: Create output directory if needed
    create_dir_all(&config.output_dir)
        .map_err(|e| format!("Failed to create output dir: {}", e))?;
    
    // Step 3: Generate backup filename
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("backup_{}.tar", timestamp);
    let output_path = config.output_dir.join(filename);
    
    // Step 4: Create tar archive
    let tar_file = File::create(&output_path)
        .map_err(|e| format!("Failed to create tar file: {}", e))?;
    
    // Step 5: Add compression wrapper
    let mut archive = match config.compression {
        CompressionType::Gzip => {
            let level = flate2::Compression::new(config.compression_level as u32);
            Builder::new(GzEncoder::new(tar_file, level))
        },
        CompressionType::Bzip2 | CompressionType::Xz | CompressionType::None=> {
            // Similar for bzip2
            let level = flate2::Compression::new(config.compression_level as u32);
            Builder::new(GzEncoder::new(tar_file, level)) // Temporary - add bzip2 later
        },
    };
    
    // Step 6: Walk directory and add files
    for entry in WalkDir::new(&config.source_path) {
        let entry = entry.map_err(|e| format!("Walkdir error: {}", e))?;
        
        // Skip excluded patterns
        if should_exclude(entry.path(), &config.exclude_patterns) {
            continue;
        }
        
        // Add file to archive
        if entry.file_type().is_file() {
            archive.append_file(entry.path(), &mut File::open(entry.path()).expect("Error"))
                .map_err(|e| format!("Failed to add file to archive: {}", e))?;
        }
    }
    
    // Step 7: Finish archive
    archive.finish()
        .map_err(|e| format!("Failed to finish archive: {}", e))?;
    
    // Step 8: Rename with compression extension
    let final_name = match config.compression {
        CompressionType::Gzip => format!("backup_{}.tar.gz", timestamp),
        CompressionType::Bzip2 => format!("backup_{}.tar.bz2", timestamp),
        CompressionType::Xz => format!("backup_{}.tar.xz", timestamp),
        CompressionType::None => format!("backup_{}.tar", timestamp),
    };
    
    // Rename file
    std::fs::rename(output_path, config.output_dir.join(final_name))
        .map_err(|e| format!("Failed to rename file: {}", e))?;
    
    Ok(())
}

fn should_exclude(path: &Path, patterns: &[String]) -> bool {
    // Simple pattern matching - improve later
    let path_str = path.to_string_lossy();
    for pattern in patterns {
        if path_str.contains(pattern) {
            return true;
        }
    }
    false
}
