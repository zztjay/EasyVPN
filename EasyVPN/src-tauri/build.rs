fn main() {
    // 打印当前工作目录
    println!("当前工作目录: {:?}", std::env::current_dir().unwrap());
    
    // 继续正常构建
    tauri_build::build();
}
