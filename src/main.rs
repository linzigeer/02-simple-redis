use tracing_subscriber::FmtSubscriber;

mod resp;

fn main() {
    // 创建一个日志订阅者
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();

    // 全局设置订阅者
    tracing::subscriber::set_global_default(subscriber).expect("设置全局默认订阅者失败");
    println!("Hello, world!");
    println!("where is the automatically hint?");
}
