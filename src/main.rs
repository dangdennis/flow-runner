#[cfg(feature = "tokio-runtime")]
use bastion::prelude::*;

use flow_sdk::prelude::*;
use std::error::Error;

#[cfg(feature = "tokio-runtime")]
use tokio;

#[cfg(feature = "tokio-runtime")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Running flow nft gateway");

    Bastion::init();

    Bastion::start();

    let supervisor = Bastion::supervisor(|sp| {
        sp
            // ...with a specific supervision strategy...
            .with_strategy(SupervisionStrategy::OneForOne)
            // ...and some supervised children groups...
            .children(|children| {
                // ...
                children.with_exec(|ctx: BastionContext| {
                    async move {
                        // ...receiving and matching messages...
                        msg! { ctx.recv().await?,
                            // ref <name> are broadcasts.
                            ref msg: &'static str => {
                                println!("with msg {}", msg);
                                let _ = poll_chain().await?
                                println!("ran flow block {}", msg);
                                ()
                            };
                            // <name> (without the ref keyword) are messages that have a unique recipient.
                            msg: &'static str => {
                                println!("with msg {}", msg)
                            };
                            // =!> refer to messages that can be replied to.
                            msg: &'static str =!> {
                                println!("with msg {}", msg)
                            };
                            // <name> that have the `_` type are catch alls
                            _: _ => ();
                        }

                        // ...

                        Ok(())
                    }
                })
            })
    })
    .expect("Couldn't create the supervisor.");

    let _ = supervisor.broadcast("poll");

    Bastion::block_until_stopped();

    Ok(())
}

async fn poll_chain() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut client = TonicHyperFlowClient::testnet().await?;
    client.ping().await?;

    let latest_block = client.latest_block(Seal::Sealed).await?;

    let block_by_id = client.block_by_id(&latest_block.id).await?;

    let block_by_height = client.block_by_height(latest_block.height).await?;

    assert_eq!(latest_block, block_by_id);
    assert_eq!(latest_block, block_by_height);

    println!("OK: {:#?}", latest_block);

    Ok(())
}
