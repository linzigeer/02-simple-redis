use anyhow::Result;
use tokio::sync::{mpsc, oneshot};

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut vec_db = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    let (tx, mut rx) = mpsc::channel::<(u8, oneshot::Sender<bool>)>(100);

    let tx1 = tx.clone();

    let task_1 = tokio::task::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        if tx1.send((50, resp_tx)).await.is_err() {
            println!("tx1 try to send value, but some error happened!");
            return;
        };

        if let Ok(result) = resp_rx.await {
            if result {
                println!("task_1 finished with success!");
            } else {
                println!("task_1 finished with failure!");
            }
        } else {
            println!("oneshot sender dropped!");
        };
    });

    let task_2 = tokio::task::spawn(async move {
        let (reps_tx, resp_rx) = oneshot::channel();
        if tx.send((100, reps_tx)).await.is_err() {
            println!("tx try to send value, but some error happened!");
            return;
        };

        if let Ok(result) = resp_rx.await {
            if result {
                println!("task_2 finished with success!");
            } else {
                println!("task_2 finished with failure!");
            }
        } else {
            println!("oneshot channel dropped!");
        }
    });

    let task_3 = tokio::task::spawn(async move {
        while let Some((value, resp_tx)) = rx.recv().await {
            println!("got value:{value}");
            vec_db[4] = value;
            println!("{:?}", vec_db);
            resp_tx.send(true).expect("Send back result failure!");
        }
    });

    task_1.await?;
    task_2.await?;
    task_3.await?;

    Ok(())
}
