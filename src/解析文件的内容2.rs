use csv::Writer;
use ipnetwork::IpNetwork;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

/* 匹配dir_path目录中，以delegated开头的文件（除了.exe或.csv外） */
fn find_matching_files(dir_path: &str) -> io::Result<Vec<PathBuf>> {
    let mut matching_files = Vec::new();
    let dir_entries = fs::read_dir(dir_path)?;
    for entry in dir_entries {
        if let Ok(entry) = entry {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.starts_with("delegated")
                    && !file_name.ends_with(".exe")
                    && !file_name.ends_with(".csv")
                {
                    matching_files.push(entry.path());
                }
            }
        }
    }

    Ok(matching_files)
}

/* 处理行内容 */
fn process_line(
    line_content: &str,
    network_writer: &mut Writer<File>,
    asn_writer: &mut Writer<File>,
) -> Result<(), Box<dyn Error>> {
    let parts: Vec<&str> = line_content.split('|').collect();
    let pipe_count = parts.len();

    if pipe_count >= 6 {
        let rir_str = parts.get(0).cloned().unwrap_or_default(); // 区域互联网注册管理机构
        let country_code = parts.get(1).cloned().unwrap_or_default();
        let date_str = parts.get(5).cloned().unwrap_or_default(); // 分配时间

        if parts[2].to_lowercase().contains("ipv") {
            let ip_str = parts.get(3).cloned().unwrap_or_default();
            let host_number_str = parts.get(4).cloned().unwrap_or_default();

            if let Ok(parsed_network) = parse_host_and_ip(&ip_str, &host_number_str) {
                write_to_csv(
                    network_writer,
                    &rir_str,
                    &parsed_network.to_string(),
                    &country_code,
                    &host_number_str,
                    &date_str,
                )?;
                println!(
                    "{},{},{},{},{}",
                    rir_str, parsed_network, host_number_str, country_code, date_str
                );
            }
        } else if parts[2].to_lowercase().contains("asn") {
            let asn_number_str = parts.get(3).cloned().unwrap_or_default();

            if let Ok(asn_number) = i32::from_str(asn_number_str) {
                write_to_asn_csv(
                    asn_writer,
                    &rir_str,
                    &country_code,
                    &format!("AS{asn_number}"),
                    &date_str,
                )?;
                println!("{},{},AS{},{}", rir_str, country_code, asn_number, date_str);
            } else {
                // asn_number 不是一个数字，可以处理错误或跳过这一行
            }
        }
    }

    Ok(())
}

/* 传入开始IP地址和主机数，计算出CIDR是多少？ */
fn parse_host_and_ip(ip_str: &str, host_number_str: &str) -> Result<IpNetwork, Box<dyn Error>> {
    // 将字符串解析为 i32
    match host_number_str.parse::<i32>() {
        Ok(parsed_number) => {
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                let mask_length = match ip {
                    IpAddr::V4(_) => 32 - parsed_number.trailing_zeros() as u8,
                    IpAddr::V6(_) => 128 - parsed_number.trailing_zeros() as u8,
                };
                return Ok(IpNetwork::new(ip, mask_length)?);
            }
        }
        Err(_) => {
            // 解析失败，处理错误
        }
    }

    Err("Invalid IP network".into())
}

fn write_to_csv(
    writer: &mut Writer<File>,
    rir_str: &str,
    network: &str,
    country_code: &str,
    host_number_str: &str,
    date_str: &str,
) -> Result<(), csv::Error> {
    writer.write_record(&[rir_str, network, country_code, host_number_str, date_str])?;
    writer.flush()?;
    Ok(())
}

fn write_to_asn_csv(
    writer: &mut Writer<File>,
    rir_str: &str,
    country_code: &str,
    asn_number_str: &str,
    date_str: &str,
) -> Result<(), csv::Error> {
    writer.write_record(&[rir_str, country_code, asn_number_str, date_str])?;
    writer.flush()?;
    Ok(())
}

fn process_file(file_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let network_output_str = format!("{}_output.csv", file_path);
    let asn_output_str = format!("{}_output(ASN).csv", file_path);

    let network_output_file = File::create(network_output_str)?;
    let asn_output_file = File::create(asn_output_str)?;

    let mut network_writer = Writer::from_writer(network_output_file);
    let mut asn_writer = Writer::from_writer(asn_output_file);

    // 写入标题行
    network_writer.write_record(&[
        "RIR",
        "Network",
        "Country Code",
        "Host Number",
        "Allocated Date",
    ])?;
    asn_writer.write_record(&["RIR", "Country Code", "ASN Number", "Allocated Date"])?;

    for line in reader.lines() {
        let line_content = line?;
        process_line(&line_content, &mut network_writer, &mut asn_writer)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let dir_path = "."; // 使用当前目录
    let matching_files = find_matching_files(dir_path)?;
    for file_path in matching_files {
        println!("\n\n操作文件：{:?}", file_path);
        process_file(file_path.to_str().unwrap())?;
    }
    println!("\n程序运行完毕！3秒后自动退出程序！");
    // 睡眠3秒
    thread::sleep(Duration::from_secs(3));
    Ok(())
}
