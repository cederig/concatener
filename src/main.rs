use clap::{Arg, Command};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let matches = Command::new("concatener")
        .version("0.1.0")
        .about("A fast command-line tool for concatenating multiple files")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file path")
                .required(true)
        )
        .arg(
            Arg::new("recursive")
                .short('r')
                .long("recursive")
                .help("Recursively search directories for files")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("inputs")
                .help("Input files, directories, or patterns to concatenate")
                .required(true)
                .num_args(1..)
        )
        .get_matches();

    let output_path = matches.get_one::<String>("output").unwrap();
    let inputs: Vec<&String> = matches.get_many::<String>("inputs").unwrap().collect();
    let recursive = matches.get_flag("recursive");

    let mut all_files = Vec::new();
    
    for input in inputs {
        let files = resolve_input_files(input, recursive)
            .with_context(|| format!("Failed to resolve input: {}", input))?;
        all_files.extend(files);
    }

    if all_files.is_empty() {
        eprintln!("Warning: No input files found to concatenate");
        return Ok(());
    }

    // Sort files for consistent ordering
    all_files.sort();
    
    concatenate_files(&all_files, output_path)
        .with_context(|| format!("Failed to concatenate files to: {}", output_path))?;

    println!("Successfully concatenated {} files to: {}", all_files.len(), output_path);
    Ok(())
}

fn resolve_input_files(input: &str, recursive: bool) -> Result<Vec<PathBuf>> {
    // Expand ~ to home directory
    let expanded_input = if input.starts_with("~/") {
        if let Some(home_dir) = std::env::var_os("HOME") {
            input.replacen("~", &home_dir.to_string_lossy(), 1)
        } else {
            input.to_string()
        }
    } else {
        input.to_string()
    };
    
    let path = Path::new(&expanded_input);
    
    // Check if it's a directory with wildcard pattern (like "dir/*.json")
    if expanded_input.contains('*') && expanded_input.contains('/') {
        // Extract directory path before the last slash
        if let Some(last_slash) = expanded_input.rfind('/') {
            let dir_path = &expanded_input[..last_slash];
            let pattern = &expanded_input[last_slash + 1..];
            let dir = Path::new(dir_path);
            
            if dir.is_dir() {
                // It's a directory with wildcard pattern
                if recursive {
                    let mut files = Vec::new();
                    collect_files_recursive_with_pattern(dir, pattern, &mut files)?;
                    Ok(files)
                } else {
                    collect_files_in_directory_with_pattern(dir, pattern, &mut vec![])
                }
            } else {
                // Not a valid directory, treat as regular wildcard
                if recursive {
                    collect_files_with_wildcard_recursive(&expanded_input)
                } else {
                    collect_files_with_wildcard(&expanded_input)
                }
            }
        } else {
            // No directory path, treat as regular wildcard
            if recursive {
                collect_files_with_wildcard_recursive(&expanded_input)
            } else {
                collect_files_with_wildcard(&expanded_input)
            }
        }
    } else if path.is_dir() {
        // Handle directory - get all files in directory
        if recursive {
            collect_files_recursive(path)
        } else {
            collect_files_in_directory(path)
        }
    } else if expanded_input.contains('*') {
        // Handle wildcard pattern (without directory path)
        if recursive {
            collect_files_with_wildcard_recursive(&expanded_input)
        } else {
            collect_files_with_wildcard(&expanded_input)
        }
    } else if path.is_file() {
        // Handle single file
        Ok(vec![path.to_path_buf()])
    } else {
        Err(anyhow::anyhow!("Input path does not exist: {}", input))
    }
}

fn collect_files_in_directory(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))? 
    {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_file() {
            files.push(entry_path);
        }
    }
    Ok(files)
}

fn collect_files_in_directory_with_pattern(dir: &Path, pattern: &str, _files: &mut Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    for entry in fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))? 
    {
        let entry = entry?;
        let entry_path = entry.path();
        
        if entry_path.is_file() {
            // Check if filename matches the pattern
            if let Some(file_name) = entry_path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    if matches_pattern(file_name_str, pattern) {
                        result.push(entry_path);
                    }
                }
            }
        }
    }
    Ok(result)
}

fn collect_files_with_wildcard(pattern: &str) -> Result<Vec<PathBuf>> {
    let final_pattern = if pattern.contains('/') {
        pattern.to_string()
    } else {
        pattern.to_string()
    };
    
    let paths = glob::glob(&final_pattern)
        .with_context(|| format!("Invalid glob pattern: {}", final_pattern))?;
    
    let mut files = Vec::new();
    for path in paths {
        let path = path.with_context(|| format!("Error reading file path"))?;
        if path.is_file() {
            files.push(path);
        }
    }
    Ok(files)
}

fn collect_files_with_wildcard_recursive(pattern: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    // If pattern contains a path, extract the directory and pattern
    if pattern.contains('/') {
        // Pattern like "src/**/*.txt" or "docs/*.md"
        let (base_dir, file_pattern) = if let Some(last_slash) = pattern.rfind('/') {
            let base_dir = &pattern[..last_slash];
            let file_pattern = &pattern[last_slash + 1..];
            (base_dir, file_pattern)
        } else {
            (".", pattern)
        };
        
        let base_path = Path::new(base_dir);
        if base_path.is_dir() {
            collect_files_recursive_with_pattern(base_path, file_pattern, &mut files)?;
        }
    } else {
        // Pattern like "*.txt" - search in current directory recursively
        collect_files_recursive_with_pattern(Path::new("."), pattern, &mut files)?;
    }
    
    Ok(files)
}

fn collect_files_recursive_with_pattern(dir: &Path, pattern: &str, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))? 
    {
        let entry = entry?;
        let entry_path = entry.path();
        
        if entry_path.is_file() {
            // Check if filename matches the pattern
            if let Some(file_name) = entry_path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    if matches_pattern(file_name_str, pattern) {
                        files.push(entry_path);
                    }
                }
            }
        } else if entry_path.is_dir() {
            // Recursively search subdirectories
            collect_files_recursive_with_pattern(&entry_path, pattern, files)?;
        }
    }
    Ok(())
}

fn matches_pattern(filename: &str, pattern: &str) -> bool {
    // Simple pattern matching - supports * wildcard
    // For more complex patterns, we could use the glob crate, but this is sufficient for basic cases
    if pattern == "*" {
        return true;
    }
    
    if pattern.starts_with('*') && pattern.ends_with('*') {
        // Contains pattern
        let middle = &pattern[1..pattern.len()-1];
        filename.contains(middle)
    } else if pattern.starts_with('*') {
        // Ends with pattern
        let suffix = &pattern[1..];
        filename.ends_with(suffix)
    } else if pattern.ends_with('*') {
        // Starts with pattern
        let prefix = &pattern[..pattern.len()-1];
        filename.starts_with(prefix)
    } else {
        // Exact match
        filename == pattern
    }
}

fn collect_files_recursive(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    for entry in fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))? 
    {
        let entry = entry?;
        let entry_path = entry.path();
        
        if entry_path.is_file() {
            files.push(entry_path);
        } else if entry_path.is_dir() {
            // Recursively collect files from subdirectory
            let sub_files = collect_files_recursive(&entry_path)?;
            files.extend(sub_files);
        }
    }
    
    Ok(files)
}

fn concatenate_files(files: &[PathBuf], output_path: &str) -> Result<()> {
    let mut output = fs::File::create(output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path))?;
    
    for (index, file_path) in files.iter().enumerate() {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {:?}", file_path))?;
        
        // Remove trailing newlines from content to avoid double newlines
        let trimmed_content = content.trim_end();
        output.write_all(trimmed_content.as_bytes())
            .with_context(|| format!("Failed to write content from file: {:?}", file_path))?;
        
        // Add newline between files (but not after the last file)
        if index < files.len() - 1 {
            writeln!(output)?;
        }
    }
    
    output.flush()
        .with_context(|| "Failed to flush output file")?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_single_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, World!")?;
        
        let files = resolve_input_files(file_path.to_str().unwrap(), false)?;
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], file_path);
        Ok(())
    }

    #[test]
    fn test_resolve_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create multiple files in directory
        fs::write(temp_dir.path().join("file1.txt"), "Content 1")?;
        fs::write(temp_dir.path().join("file2.txt"), "Content 2")?;
        fs::write(temp_dir.path().join("file3.txt"), "Content 3")?;
        
        // Create a subdirectory (should be ignored when not recursive)
        fs::create_dir(temp_dir.path().join("subdir"))?;
        
        let files = resolve_input_files(temp_dir.path().to_str().unwrap(), false)?;
        assert_eq!(files.len(), 3);
        
        // Check that all expected files are present
        let file_names: Vec<String> = files.iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        assert!(file_names.contains(&"file1.txt".to_string()));
        assert!(file_names.contains(&"file2.txt".to_string()));
        assert!(file_names.contains(&"file3.txt".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_resolve_wildcard() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create multiple files with different extensions
        fs::write(temp_dir.path().join("test1.txt"), "Content 1")?;
        fs::write(temp_dir.path().join("test2.txt"), "Content 2")?;
        fs::write(temp_dir.path().join("other.log"), "Log content")?;
        
        // Change to temp directory for wildcard testing
        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(temp_dir.path())?;
        
        let files = resolve_input_files("*.txt", false)?;
        assert_eq!(files.len(), 2);
        
        // Restore original directory
        std::env::set_current_dir(original_dir)?;
        Ok(())
    }

    #[test]
    fn test_concatenate_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create test files
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        let output = temp_dir.path().join("output.txt");
        
        fs::write(&file1, "Hello")?;
        fs::write(&file2, "World")?;
        
        concatenate_files(&[file1.clone(), file2.clone()], output.to_str().unwrap())?;
        
        let result = fs::read_to_string(&output)?;
        assert_eq!(result, "Hello\nWorld");
        Ok(())
    }

    #[test]
    fn test_concatenate_single_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        let file1 = temp_dir.path().join("file1.txt");
        let output = temp_dir.path().join("output.txt");
        
        fs::write(&file1, "Single content")?;
        
        concatenate_files(&[file1.clone()], output.to_str().unwrap())?;
        
        let result = fs::read_to_string(&output)?;
        assert_eq!(result, "Single content");
        Ok(())
    }

    #[test]
    fn test_nonexistent_file() {
        let result = resolve_input_files("/nonexistent/file.txt", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let files = resolve_input_files(temp_dir.path().to_str().unwrap(), false)?;
        assert_eq!(files.len(), 0);
        Ok(())
    }

    #[test]
    fn test_recursive_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create files in root directory
        fs::write(temp_dir.path().join("root1.txt"), "Root content 1")?;
        fs::write(temp_dir.path().join("root2.txt"), "Root content 2")?;
        
        // Create subdirectory with files
        fs::create_dir(temp_dir.path().join("subdir1"))?;
        fs::write(temp_dir.path().join("subdir1").join("sub1.txt"), "Sub content 1")?;
        fs::write(temp_dir.path().join("subdir1").join("sub2.txt"), "Sub content 2")?;
        
        // Create nested subdirectory
        fs::create_dir(temp_dir.path().join("subdir1").join("nested"))?;
        fs::write(temp_dir.path().join("subdir1").join("nested").join("nested.txt"), "Nested content")?;
        
        // Create another subdirectory
        fs::create_dir(temp_dir.path().join("subdir2"))?;
        fs::write(temp_dir.path().join("subdir2").join("sub3.txt"), "Sub content 3")?;
        
        // Test recursive collection
        let files = resolve_input_files(temp_dir.path().to_str().unwrap(), true)?;
        assert_eq!(files.len(), 6);
        
        // Check that files from all directories are included
        let file_names: Vec<String> = files.iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        assert!(file_names.contains(&"root1.txt".to_string()));
        assert!(file_names.contains(&"root2.txt".to_string()));
        assert!(file_names.contains(&"sub1.txt".to_string()));
        assert!(file_names.contains(&"sub2.txt".to_string()));
        assert!(file_names.contains(&"nested.txt".to_string()));
        assert!(file_names.contains(&"sub3.txt".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_non_recursive_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create files in root directory
        fs::write(temp_dir.path().join("root1.txt"), "Root content 1")?;
        
        // Create subdirectory with files
        fs::create_dir(temp_dir.path().join("subdir1"))?;
        fs::write(temp_dir.path().join("subdir1").join("sub1.txt"), "Sub content 1")?;
        
        // Test non-recursive collection (should only get root files)
        let files = resolve_input_files(temp_dir.path().to_str().unwrap(), false)?;
        assert_eq!(files.len(), 1);
        
        let file_names: Vec<String> = files.iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        assert!(file_names.contains(&"root1.txt".to_string()));
        assert!(!file_names.contains(&"sub1.txt".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_recursive_wildcard() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create files in root directory
        fs::write(temp_dir.path().join("root1.txt"), "Root content 1")?;
        fs::write(temp_dir.path().join("root2.log"), "Root log")?;
        
        // Create subdirectory with files
        fs::create_dir(temp_dir.path().join("subdir1"))?;
        fs::write(temp_dir.path().join("subdir1").join("sub1.txt"), "Sub content 1")?;
        fs::write(temp_dir.path().join("subdir1").join("sub2.log"), "Sub log")?;
        
        // Create nested subdirectory
        fs::create_dir(temp_dir.path().join("subdir1").join("nested"))?;
        fs::write(temp_dir.path().join("subdir1").join("nested").join("nested.txt"), "Nested content")?;
        
        // Change to temp directory for wildcard testing
        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(temp_dir.path())?;
        
        // Test recursive wildcard for .txt files
        let files = resolve_input_files("*.txt", true)?;
        assert_eq!(files.len(), 3); // root1.txt, sub1.txt, nested.txt
        
        // Test recursive wildcard for .log files
        let log_files = resolve_input_files("*.log", true)?;
        assert_eq!(log_files.len(), 2); // root2.log, sub2.log
        
        // Test non-recursive wildcard (should only get root files)
        let non_recursive_files = resolve_input_files("*.txt", false)?;
        assert_eq!(non_recursive_files.len(), 1); // only root1.txt
        
        // Restore original directory
        std::env::set_current_dir(original_dir)?;
        Ok(())
    }

    #[test]
    fn test_recursive_wildcard_with_path() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create directory structure
        fs::create_dir(temp_dir.path().join("src"))?;
        fs::write(temp_dir.path().join("src").join("main.rs"), "Main code")?;
        fs::write(temp_dir.path().join("src").join("utils.rs"), "Utils code")?;
        
        fs::create_dir(temp_dir.path().join("src").join("subdir"))?;
        fs::write(temp_dir.path().join("src").join("subdir").join("module.rs"), "Module code")?;
        
        fs::create_dir(temp_dir.path().join("docs"))?;
        fs::write(temp_dir.path().join("docs").join("readme.md"), "Documentation")?;
        
        // Test recursive wildcard with path
        let pattern = format!("{}/*.rs", temp_dir.path().join("src").display());
        let files = resolve_input_files(&pattern, true)?;
        assert_eq!(files.len(), 3); // main.rs, utils.rs, module.rs
        
        Ok(())
    }

    #[test]
    fn test_pattern_matching() {
        assert!(matches_pattern("test.txt", "*.txt"));
        assert!(matches_pattern("test.txt", "test*"));
        assert!(matches_pattern("test.txt", "*txt"));
        assert!(matches_pattern("test.txt", "*test*"));
        assert!(matches_pattern("test.txt", "test.txt"));
        assert!(matches_pattern("test.txt", "*"));
        
        assert!(!matches_pattern("test.txt", "*.log"));
        assert!(!matches_pattern("test.txt", "other*"));
        assert!(!matches_pattern("test.txt", "*other"));
        assert!(!matches_pattern("test.txt", "other.txt"));
    }
}
