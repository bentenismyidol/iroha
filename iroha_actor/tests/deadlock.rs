#![allow(clippy::unwrap_used)]

use iroha_actor::prelude::*;

#[derive(Default, Debug)]
struct DeadlockActor(Option<Addr<Self>>);
struct Msg;
struct Address<A: Actor>(Addr<A>);

impl Message for Msg {
    type Result = ();
}
impl<A: Actor> Message for Address<A> {
    type Result = ();
}
impl Actor for DeadlockActor {}

#[async_trait::async_trait]
impl Handler<Msg> for DeadlockActor {
    type Result = ();
    async fn handle(&mut self, context: &mut Context<Self>, _: Msg) {
        if let Some(addr) = &self.0 {
            let _ = addr.send(Msg, Some(context.addr().clone())).await;
        }
    }
}

#[async_trait::async_trait]
impl Handler<Address<Self>> for DeadlockActor {
    type Result = ();
    async fn handle(&mut self, _: &mut Context<Self>, Address(addr): Address<Self>) {
        self.0 = Some(addr);
    }
}

/// Basic deadlock test.
#[cfg(feature = "deadlock_detection")]
#[tokio::test(flavor = "multi_thread")]
#[should_panic]
async fn async_test() {
    let actor1 = DeadlockActor::init_default();
    let actor2 = DeadlockActor::init_default();
    let addr1 = actor1.address().clone();
    let addr2 = actor2.address().clone();
    actor1.start().await;
    actor2.start().await;
    addr1
        .send(Address(addr2.clone()), Some(addr1.clone()))
        .await
        .unwrap();
    addr2
        .send(Address(addr1.clone()), Some(addr2.clone()))
        .await
        .unwrap();
    addr1.send(Msg, Some(addr2)).await.unwrap();
    unreachable!()
}