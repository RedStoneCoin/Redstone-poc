//! cli process

use super::*;
use crate::blockchain::*;
use crate::server::*;
use crate::transaction::*;
use crate::utxoset::*;
use crate::wallets::*;
use bitcoincash_addr::Address;
use clap::{App, Arg};
use std::process::exit;

pub struct Cli {}

impl Cli {
    pub fn new() -> Cli {
        Cli {}
    }

    pub fn run(&mut self) -> Result<()> {
        info!("run app");
        let matches = App::new("Redstone cb")
            .version("0.1")
            .author("Redstone Developers")
            .about("Implementation of redstone protocol ")
            
            .subcommand(App::new("printchain").about("print all the chain blocks"))
            .subcommand(App::new("createwallet").about("create a wallet"))
            .subcommand(App::new("listaddresses").about("list all addresses"))
            .subcommand(App::new("reindex").about("reindex UTXO"))
            .subcommand(
                App::new("startnode")
                    .about("start the node server")
                    .arg(Arg::from_usage("<port> 'the port server bind to locally'")),
            )
            .subcommand(
                App::new("startminer")
                    .about("start the minner server")
                    .arg(Arg::from_usage("<port> 'the port server bind to locally'"))
                    .arg(Arg::from_usage("<address> 'wallet address'")),
            )
            .subcommand(
                App::new("getbalance")
                    .about("get balance in the blockchain")
                    .arg(Arg::from_usage(
                        "<address> 'The address to get balance for'",
                    )),
            )
            .subcommand(App::new("createblockchain").about("create blockchain").arg(
                Arg::from_usage("<address> 'The address to send genesis block reward to'"),
            ))
            .subcommand(
                App::new("send")
                    .about("send in the blockchain")
                    .arg(Arg::from_usage("<from> 'Source wallet address'"))
                    .arg(Arg::from_usage("<to> 'Destination wallet address'"))
                    .arg(Arg::from_usage("<amount> 'Amount to send'"))
                    .arg(Arg::from_usage("<chain> 'Send to what chain'"))
                    .arg(Arg::from_usage("-m --mine 'the from address mine immediately'",)),
            )
            .get_matches();

        if let Some(ref matches) = matches.subcommand_matches("getbalance") {
            if let Some(address) = matches.value_of("address") {
                let balance = cmd_get_balance(address)?;
                println!("Balance: {}\n", balance);
            }
        } else if let Some(_) = matches.subcommand_matches("createwallet") {
            println!("address: {}", cmd_create_wallet()?);
        } else if let Some(_) = matches.subcommand_matches("printchain") {
            println!("Chain 1:");
            cmd_print_chain()?;
            println!("Chain 2:");
            cmd_print_chain2()?;
        } else if let Some(_) = matches.subcommand_matches("reindex") {
            let count = cmd_reindex()?;
            println!("Done! There are {} transactions in the UTXO set.", count);
            println!("Chain 2:");
            let count1 = cmd_reindex1()?;
            println!("Done! There are {} transactions in the UTXO set.", count1);
        } else if let Some(_) = matches.subcommand_matches("listaddresses") {
            cmd_list_address()?;
        } else if let Some(ref matches) = matches.subcommand_matches("createblockchain") {
            if let Some(address) = matches.value_of("address") {
                cmd_create_blockchain(address)?;
                cmd_create_blockchain1(address)?;
            }
        } else if let Some(ref matches) = matches.subcommand_matches("send") {
            let from = if let Some(address) = matches.value_of("from") {
                address
            } else {
                println!("from not supply!: usage\n{}", matches.usage());
                exit(1)
            };
            let to = if let Some(address) = matches.value_of("to") {
                address
            } else {
                println!("to not supply!: usage\n{}", matches.usage());
                exit(1)
            };
            let amount: i32 = if let Some(amount) = matches.value_of("amount") {
                amount.parse()?

            } else {
                println!("amount in send not supply!: usage\n{}", matches.usage());
                exit(1)
            };
            let chain: i32 = if let Some(chain) = matches.value_of("chain") {
                chain.parse()?
            } else {
                println!("Chain is bad!: usage\n{}", matches.usage());
                exit(1)
            };
            if matches.is_present("mine") {
                cmd_send(from, to, amount, true, chain)?;
            } else {
                cmd_send(from, to, amount, false, chain)?;
            }
        } else if let Some(ref matches) = matches.subcommand_matches("startnode") {
            if let Some(port) = matches.value_of("port") {
                println!("Starting node!");
                let bc = Blockchain::new()?;
                let utxo_set = UTXOSet { blockchain: bc };
                let bc1 = Blockchain::new2()?;
                let utxo_set1 = UTXOSet { blockchain: bc1 };
                let server = Server::new(port, "", utxo_set)?;
                let server1 = Server::new(port, "", utxo_set1)?;
                println!("Starting node theards!");
                server.start_server()?;
                server1.start_server()?;
                
            }
        } else if let Some(ref matches) = matches.subcommand_matches("startminer") {
            let address = if let Some(address) = matches.value_of("address") {
                address
            } else {
                println!("address not supply!: usage\n{}", matches.usage());
                exit(1)
            };
            let port = if let Some(port) = matches.value_of("port") {
                port
            } else {
                println!("port not supply!: usage\n{}", matches.usage());
                exit(1)
            };
            println!("Start miner node...");
            let bc = Blockchain::new()?;
            let utxo_set = UTXOSet { blockchain: bc };
            let server = Server::new(port, address, utxo_set)?;
            server.start_server()?;
            let bc1 = Blockchain::new2()?;
            let utxo_set1 = UTXOSet { blockchain: bc1 };
            let server1 = Server::new(port, address, utxo_set1)?;
            server1.start_server()?;
            //this should start node on both server if not we will start in in a thread

        }

        Ok(())
    }
}

fn cmd_send(from: &str, to: &str, amount: i32, mine_now: bool, chain: i32) -> Result<()> {
    let bc = Blockchain::new()?;
    let bc1 = Blockchain::new2()?;
    let mut utxo_set = UTXOSet { blockchain: bc };
    let mut utxo_set1 = UTXOSet { blockchain: bc1 };

    let wallets = Wallets::new()?;
    let wallet = wallets.get_wallet(from).unwrap();
    let tx = Transaction::new_UTXO(wallet, to, amount, &utxo_set)?;
    let tx1 = Transaction::new_UTXO(wallet, to, amount, &utxo_set1)?;

    if mine_now {
        match chain {
            2 => {
            // handle chain 2
            println!("Sending to chain 2");
            let cbtx = Transaction::new_coinbase(from.to_string(), String::from("reward!"))?;
            let new_block = utxo_set1.blockchain.mine_block(vec![cbtx, tx1])?;
            utxo_set1.update(&new_block,2)?;
            println!("success!");
            }
            1 => {
            // handle chain 1
            println!("Sending to chain 1");
            let cbtx = Transaction::new_coinbase(from.to_string(), String::from("reward!"))?;
            let new_block = utxo_set.blockchain.mine_block(vec![cbtx, tx])?;
            utxo_set.update(&new_block,1)?;
            println!("success!");
            }_ => {
               println!("Unknown chain index: {}", chain);
            }
          };
    

    } else {
        match chain {
            1 => {
            // handle chain 1
            Server::send_transaction(&tx, utxo_set)?;
            }
            2 => {
            // handle chain 1
            Server::send_transaction(&tx1, utxo_set1)?;

            }
            
            _ => {
               println!("Unknown chain index: {}", chain);
            }
          };


    }

   
    Ok(())
}


fn cmd_create_wallet() -> Result<String> {
    let mut ws = Wallets::new()?;
    let address = ws.create_wallet();
    ws.save_all()?;
    Ok(address)
}

fn cmd_reindex() -> Result<i32> {
    let bc = Blockchain::new()?;
    let utxo_set = UTXOSet { blockchain: bc };

    utxo_set.reindex()?;
    utxo_set.count_transactions()
}
fn cmd_reindex1() -> Result<i32> {
    let bc = Blockchain::new2()?;
    let utxo_set = UTXOSet { blockchain: bc };

    utxo_set.reindex()?;
    utxo_set.count_transactions()
}
fn cmd_create_blockchain(address: &str) -> Result<()> {
    let address = String::from(address);
    let bc = Blockchain::create_blockchain(address)?;

    let utxo_set = UTXOSet { blockchain: bc };
    utxo_set.reindex()?;
    println!("create blockchain");
    Ok(())
}
fn cmd_create_blockchain1(address: &str) -> Result<()> {
    let address = String::from(address);
    let bc = Blockchain::create_blockchain1(address)?;

    let utxo_set = UTXOSet { blockchain: bc };
    utxo_set.reindex()?;
    println!("create blockchain 2");
    Ok(())
}

fn cmd_get_balance(address: &str) -> Result<i32> {
    let pub_key_hash = Address::decode(address).unwrap().body;
    let bc = Blockchain::new()?;
    let utxo_set = UTXOSet { blockchain: bc };
    let utxos = utxo_set.find_UTXO(&pub_key_hash)?;

    let mut balance = 0;
    for out in utxos.outputs {
        balance += out.value;
    }
    Ok(balance)
}

fn cmd_print_chain() -> Result<()> {
    info!("chain 1");
    let bc = Blockchain::new()?;
    for b in bc.iter() {
        println!("{:#?}", b);
    }
    Ok(())
}
fn cmd_print_chain2() -> Result<()> {

    info!("chain 2");
    let bc1 = Blockchain::new2()?;
    for b in bc1.iter() {
        println!("{:#?}", b);
    }
    Ok(())
}
fn cmd_list_address() -> Result<()> {
    let ws = Wallets::new()?;
    let addresses = ws.get_all_addresses();
    println!("addresses: ");
    for ad in addresses {
        println!("{}", ad);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_locally() {
        let addr1 = cmd_create_wallet().unwrap();
        let addr2 = cmd_create_wallet().unwrap();
        cmd_create_blockchain(&addr1).unwrap();

        let b1 = cmd_get_balance(&addr1).unwrap();
        let b2 = cmd_get_balance(&addr2).unwrap();
        assert_eq!(b1, 10);
        assert_eq!(b2, 0);

        cmd_send(&addr1, &addr2, 5, true, 1).unwrap();

        let b1 = cmd_get_balance(&addr1).unwrap();
        let b2 = cmd_get_balance(&addr2).unwrap();
        assert_eq!(b1, 15);
        assert_eq!(b2, 5);

        cmd_send(&addr2, &addr1, 15, true, 1).unwrap_err();
        let b1 = cmd_get_balance(&addr1).unwrap();
        let b2 = cmd_get_balance(&addr2).unwrap();
        assert_eq!(b1, 15);
        assert_eq!(b2, 5);
    }
}
