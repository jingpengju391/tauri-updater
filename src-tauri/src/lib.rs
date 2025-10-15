// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri_plugin_updater::UpdaterExt;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                update(handle).await.unwrap();
            });
            Ok(())
        })
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


/// 检查、下载并安装更新
/// 注意：该函数通常会在主进程或命令调用时触发
async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
  // 检查服务器上是否有新版本（会自动访问 latest.json）
  if let Some(update) = app.updater()?.check().await? {
    // update 对象包含了本次更新的所有信息：
    // update.version — 新版本号
    // update.date — 发布时间
    // update.body — 更新说明（notes 字段）
    // update.download_url — 更新包的下载地址

    println!("🔍 检测到新版本: {}", update.version);
    println!(
      "📝 更新内容:\n{}",
      update.body.as_deref().unwrap_or("（没有更新说明）")
    ); // 这里就是 latest.json 里的 notes！

    let mut downloaded = 0;

    // 下载 + 安装（合并为一步）
    update
      .download_and_install(
        |chunk_length, content_length| {
          downloaded += chunk_length;
          println!("⬇️ 下载进度: {downloaded}/{:?}", content_length);
        },
        || {
          println!("✅ 下载完成，开始安装...");
        },
      )
      .await?;

    println!("🚀 更新安装完成！");
    app.restart();
  } else {
    println!("当前已是最新版本 ✅");
  }

  Ok(())
}
