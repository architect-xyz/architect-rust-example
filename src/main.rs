use anyhow::{anyhow, Result};
use architect_sdk::{Common, oms::OmsClient, symbology::*};
use architect_api::{Dir, orderflow::*};
use rust_decimal_macros::dec;

#[tokio::main]
async fn main() -> Result<()> {
    // Programs that run against the Architect core must load the same configuration file
    // that the core is running with.  The config file can be explicitly specified, or
    // found in an OS-default location.
    //
    //   let common = Common::load("/path/to/config.yml")?;
    //   let common = Common::load_default()?;
    //
    // For this example, we run against the Docker demo environment, whose configs are
    // copied into this repo for convenience.
    let common = Common::load("./env/docker/architect/config.yml").await?;
    // Architect uses unified, normalized symbology.  The Docker demo environment comes
    // with a symbology service loaded with OKX symbols, which we'll use in this example.
    common.start_symbology(false).await;
    // Create an Architect OMS client and connect using the default ChannelAuthority and Oms;
    // if you're running against the Docker demo environment, a core with those defaults will
    // already be set up for you.
    let mut oms = OmsClient::connect(&common, None, None).await?;
    let oid = oms.orderflow.next_order_id();
    let market = Market::get("BTC Crypto/USD*OKX/DIRECT").ok_or_else(|| anyhow!("no symbol"))?;
    oms.orderflow.send(OrderflowMessage::Order(Order {
        id: oid,
        market: market.id,
        dir: Dir::Buy,
        price: dec!(10000),
        quantity: dec!(0.0001),
        account: None,
    }))?;
    'outer: loop {
        let updates = oms.next().await?;
        for update in updates {
            println!("OmsOrderUpdate: {:?}", update);
            if update.state.contains(OrderStateFlags::Out) {
                println!("order is out");
                break 'outer;
            }
        }
    }
    Ok(())
}
