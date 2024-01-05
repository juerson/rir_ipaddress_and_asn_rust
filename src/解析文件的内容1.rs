use csv::Writer;
use ipnetwork::IpNetwork;
use std::fs;
use std::fs::File;
use std::io::{self, prelude::*};
use std::net::IpAddr;
use std::path::PathBuf;

fn find_matching_files(dir_path: &str) -> io::Result<Vec<PathBuf>> {
    let mut matching_files = Vec::new();

    let dir_entries = fs::read_dir(dir_path)?;

    for entry in dir_entries {
        if let Ok(entry) = entry {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.starts_with("delegated") && !file_name.ends_with(".exe") {
                    matching_files.push(entry.path());
                }
            }
        }
    }

    Ok(matching_files)
}

fn main() -> io::Result<()> {
    let dir_path = "."; // 使用当前目录作为示例

    let matching_files = find_matching_files(dir_path)?;

    for file_path in matching_files {
        // 在这里可以使用匹配的文件路径进行其他操作
        let _file = File::open(&file_path)?;

        println!("Opened file: {:?}", file_path);
    }
    let file = File::open("delegated-apnic-extended-latest")?;
    let reader = io::BufReader::new(file);

    // 限制读取的行数
    //    let mut line_count = 0;
    //    let max_lines = 200; // 你想要的最大行数

    // 打开或创建 CSV 文件
    let mut writer = Writer::from_path("output.csv")?;
    // 写入标题行
    writer.write_record(&["Network", "Country Code", "Host Number"])?;
    for line in reader.lines() {
        let line_content = line?;

        // 计算 "|" 字符的数量
        let pipe_count = line_content.matches('|').count();

        // 检查是否有至少 6 个 "|" 且以 "apnic" 开头
        if pipe_count >= 5
            && (line_content.to_lowercase().contains("ipv4")
                || line_content.to_lowercase().contains("ipv6"))
        {
            // 以 "|" 分割字符串
            let parts: Vec<&str> = line_content.split('|').collect();
            // 获取元素的值
            let country_code = parts.get(1).cloned().unwrap_or_default();
            let _ip_version = parts.get(2).cloned().unwrap_or_default();
            let ip_str = parts.get(3).cloned().unwrap_or_default();
            let host_number_str = parts.get(4).cloned().unwrap_or_default();

            // 定义 IpNetwork 变量
            let mut network: Option<IpNetwork> = None;
            // 尝试将字符串解析为 i32
            match host_number_str.parse::<i32>() {
                Ok(parsed_number) => {
                    // 尝试将字符串解析为 IpAddr
                    if let Ok(ip) = ip_str.parse::<IpAddr>() {
                        // 获取掩码长度
                        let mask_length = match ip {
                            IpAddr::V4(_) => 32 - parsed_number.trailing_zeros() as u8,
                            IpAddr::V6(_) => 128 - parsed_number.trailing_zeros() as u8,
                        };
                        // 创建 IpNetwork
                        network =
                            Some(IpNetwork::new(ip, mask_length).expect("Invalid IP network"));
                    } else {
                        // 无效 IP 地址
                    }
                }
                Err(_) => {
                    // 解析失败，处理错误
                }
            }

            // 在此处可以使用 network 变量
            if let Some(network) = &network {
                // 将值写入 CSV 文件
                writer.write_record(&[&network.to_string(), country_code, host_number_str])?;
                writer.flush()?;
                println!("{},{},{}", network, host_number_str, country_code)
            } else {
                // network不可用
            }
        }
        // 增加计数器
        //        line_count += 1;
        // 检查是否达到最大行数
        //        if line_count >= max_lines {
        //            break;
        //        }
    }

    Ok(())
}
