use std::fs::File;
use std::io::Write;
use url::Url;

async fn download_and_save(url_str: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 解析URL字符串
    let url = Url::parse(url_str)?;

    // 获取路径部分
    if let Some(path_segments) = url.path_segments() {
        // 获取路径的最后一部分
        if let Some(last_segment) = path_segments.last() {
            println!("Downloading {}...", last_segment);

            // 构建带有路径的URL，以确保请求的URL包含路径信息
            let full_url = url.join(last_segment)?;

            // 发起一个GET请求
            let response = reqwest::get(full_url).await?;

            // 检查响应的状态码
            if response.status().is_success() {
                // 从响应中获取响应体
                let body = response.bytes().await?;

                // 打开或创建一个本地文件
                let mut file = File::create(last_segment.to_string())?;

                // 将响应体写入文件
                file.write_all(&body)?;

                println!("文件：{} 下载成功!", last_segment);
            } else {
                println!(
                    "文件：{} 下载失败！响应状态情况：{}",
                    last_segment,
                    response.status()
                );
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let urls = vec![
        "https://ftp.ripe.net/pub/stats/apnic/delegated-apnic-extended-latest",
        "https://ftp.ripe.net/pub/stats/ripencc/delegated-ripencc-extended-latest",
        "https://ftp.ripe.net/pub/stats/arin/delegated-arin-extended-latest",
        "https://ftp.ripe.net/pub/stats/lacnic/delegated-lacnic-extended-latest",
        "https://ftp.ripe.net/pub/stats/afrinic/delegated-afrinic-extended-latest",
    ];

    // 创建一个向量来存储任务的连接句柄
    let mut handles = vec![];

    // 为每个 URL 生成一个任务
    for url in urls {
        let handle = tokio::spawn(download_and_save(url));
        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        if let Err(e) = handle.await {
            eprintln!("Error downloading resource: {:?}", e);
        }
    }

    Ok(())
}
