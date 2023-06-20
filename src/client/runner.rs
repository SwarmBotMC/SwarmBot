use std::{
    cell::RefCell,
    default::default,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Context;
use interfaces::{types::Selection2D, Attack, CommandData, GoTo};
use tokio::sync::Notify;
use tokio_stream::{Stream, StreamExt};

use crate::{
    bootstrap::BotConnection,
    client::{
        bot::{run_threaded, ActionState, Bot},
        commands::CommandReceiver,
        processor::SimpleInterfaceIn,
        state::{
            global::{mine_alloc::MinePreference, GlobalState},
            local::LocalState,
        },
        tasks::{
            attack_entity::AttackEntity, lazy_stream::LazyStream, mine_region::MineRegion,
            navigate::BlockTravelTask,
        },
    },
    protocol::{EventQueue, Login, Minecraft},
};

/// A wrapper around a raw pointer of [`GlobalState`] that is sync
struct SyncGlobal(*const GlobalState);

impl SyncGlobal {
    /// # Safety
    ///
    /// Access the state
    const unsafe fn state(&self) -> &GlobalState {
        // SAFETY:
        unsafe { &*self.0 }
    }
}

struct SyncLocal((*mut LocalState, *mut ActionState));

/// Safety:
/// TODO
unsafe impl Sync for SyncGlobal {}

/// Safety:
/// TODO
unsafe impl Send for SyncGlobal {}

/// Safety:
/// TODO
unsafe impl Sync for SyncLocal {}

/// Safety:
/// TODO
unsafe impl Send for SyncLocal {}

pub type Logins<T> = Rc<RefCell<Vec<Login<<T as Minecraft>::Queue, <T as Minecraft>::Interface>>>>;

/// Runs the game loop and holds all bots.
pub struct Runner<T: Minecraft> {
    /// logins that are about to be established
    pending_logins: Logins<T>,

    /// the global state of the program containing chunks and global config
    global_state: GlobalState,

    command_receiver: CommandReceiver,

    /// the bots created by pending logins
    bots: Vec<Bot<T::Queue, T::Interface>>,

    /// An id counter that increases for each bot. Used as a unique identifier.
    id_on: u32,
}

/// Runner launch options
pub struct RunnerOptions {
    /// The amount of milliseconds to wait between logging in successive users
    pub delay_ms: u64,
    /// the websocket port
    pub ws_port: u16,
}

impl<T: Minecraft + 'static> Runner<T> {
    /// Start the runner process
    pub async fn run(
        connections: impl Stream<Item = BotConnection> + 'static,
        opts: RunnerOptions,
    ) -> anyhow::Result<()> {
        let mut runner = Self::init(connections, opts).await?;
        runner.game_loop().await;
        Ok(())
    }

    /// Initialize the runner. Go through the handshake process for each
    /// [`Connection`]
    async fn init(
        connections: impl Stream<Item = BotConnection> + 'static,
        opts: RunnerOptions,
    ) -> anyhow::Result<Self> {
        let RunnerOptions { delay_ms, ws_port } = opts;

        // commands received over websocket (typically forge mod)
        let commands = CommandReceiver::init(ws_port).await?;

        let pending_logins = Self::login_all(connections, delay_ms);

        Ok(Self {
            pending_logins,
            global_state: GlobalState::init(),
            command_receiver: commands,
            bots: Vec::new(),
            id_on: 0,
        })
    }

    /// start the login process for all players
    ///
    /// - return pending logins
    fn login_all(
        connections: impl Stream<Item = BotConnection> + 'static,
        delay_millis: u64,
    ) -> Logins<T> {
        let mut connections = Box::pin(connections);
        let pending_logins = Rc::new(RefCell::new(Vec::new()));

        {
            let pending_logins = pending_logins.clone();

            // login task for all users
            tokio::task::spawn_local(async move {
                while let Some(connection) = connections.next().await {
                    let logins = pending_logins.clone();

                    // login task for an individual user
                    tokio::task::spawn_local(async move {
                        println!("Starting login of {}", connection.bot.username());
                        let username = connection.bot.username().to_string();
                        let login = match T::login(connection).await {
                            Ok(res) => {
                                println!("Finished logging in {username}");
                                res
                            }
                            Err(err) => {
                                println!("Error logging in {username} -- {err}");
                                return;
                            }
                        };
                        logins.borrow_mut().push(login);
                    });

                    // if we want a delay between logging in
                    tokio::time::sleep(Duration::from_millis(delay_millis)).await;
                }
            });
        }

        pending_logins
    }

    pub async fn game_loop(&mut self) {
        let mut previous_goal = Instant::now();

        // a game loop repeating every 50 ms
        loop {
            let end_by = previous_goal + Duration::from_millis(50);
            self.game_iter(end_by).await;
            tokio::time::sleep_until(tokio::time::Instant::from_std(end_by)).await;
            let now = Instant::now();
            let difference = now - end_by;
            let millis_off = difference.as_millis();

            // log if we are wayyyy off
            if millis_off > 100 {
                println!("off by {millis_off}ms");
            }

            previous_goal = end_by;
        }
    }

    async fn game_iter(&mut self, end_by: Instant) {
        let old_count = self.bots.len();
        // first step: removing disconnected clients
        self.remove_disconnected();

        // second step: turning pending logins into clients
        self.pending_logins_to_client();

        let new_count = self.bots.len();

        // log clients if they have changed
        if new_count != old_count {
            println!("{new_count} clients");
        }

        // process pending commands (from forge mod)
        self.process_forge_mod_commands();

        // fourth step: process packets from game loop
        self.process_incoming_minecraft_packets();

        // fifth step: process packets from game loop
        self.run_expensive_tasks_multithreaded(end_by).await;
    }

    /// remove disconnected clients
    fn remove_disconnected(&mut self) {
        self.bots.retain(|client| !client.state.disconnected);
    }

    /// turn pending logins into clients that are controller by the [`Runner`].
    fn pending_logins_to_client(&mut self) {
        let mut logins = self.pending_logins.borrow_mut();

        for login in logins.drain(..) {
            let Login { queue, out, info } = login;

            let client = Bot {
                state: LocalState::new(self.id_on, info),
                actions: default(),
                queue,
                out,
            };
            self.id_on += 1;
            self.bots.push(client);
        }
    }

    /// process pending commands (generally from forge mod but more generally
    /// from a websocket)
    fn process_forge_mod_commands(&mut self) {
        while let Ok(command) = self.command_receiver.pending.try_recv() {
            if let Err(err) = self.process_command(command) {
                println!("Error processing command: {err}");
            }
        }
    }

    fn process_incoming_minecraft_packets(&mut self) {
        for bot in &mut self.bots {
            let mut processor = SimpleInterfaceIn::new(
                &mut bot.state,
                &mut bot.actions,
                &mut self.global_state,
                &mut bot.out,
            );

            // protocol-specific logic. Translates input packets and sends to processor
            bot.queue.flush(&mut processor);

            // fifth step: general sync logic that isn't dependent on protocol
            // implementation
            bot.run_sync(&mut self.global_state);
        }
    }

    /// launch expensive tasks (for instance A*) in multiple threads and
    /// `.await` until either:
    ///
    /// 1. all tasks are completed
    /// 2. we have reached the max processing time before the game loop starts
    /// again
    ///
    /// This will allow all bots to access `&GlobalState` and `&mut LocalState`
    /// as the [`GlobalState`] as shared cross-thread, whereas the
    /// [`LocalState`] will only be accessed by one thread.
    ///
    /// This means that per-bot calculations (i.e., A* for one bot) will be done
    /// sequentially, whereas cross-bot calculations (i.e., A* for bot A and
    /// A* bot B) will be done concurrently.
    async fn run_expensive_tasks_multithreaded(&mut self, end_by: Instant) {
        // sixth step: run multi-threaded environment for the rest of the game loop.
        // GlobalState will be read-only and LocalState will be mutable
        let thread_loop_end = Arc::new(Notify::new());

        {
            let thread_loop_end = thread_loop_end.clone();

            // We have to do unsafe stuff here because Rust requires a 'static lifetime for
            // threads. However, in this case we know that the thread (task)
            // will stop by the end of this function, so we can coerce the
            // lifetimes of &GlobalState and &mut LocalState to be 'static. This is overall
            // pretty safe as it still requires the states to be Send+Sync, so
            // it is hard to make errors.
            let global_state_sync = SyncGlobal(&self.global_state);
            let states_sync: Vec<_> = self
                .bots
                .iter_mut()
                .map(|bot| (&mut bot.state, &mut bot.actions))
                .map(|(state, actions)| (state as *mut LocalState, actions as *mut ActionState))
                .map(SyncLocal)
                .collect();

            rayon::spawn(move || {
                // Safety:
                // TODO
                let global_state = unsafe { global_state_sync.state() };
                let states_sync = states_sync;
                rayon::scope(|s| {
                    for state_sync in states_sync {
                        let (state, actions) = state_sync.0;
                        // Safety:
                        // TODO
                        let (state, actions) = unsafe { (&mut *state, &mut *actions) };

                        s.spawn(move |inner_scope| {
                            run_threaded(inner_scope, state, actions, global_state, end_by);
                        });
                    }
                });

                // when all tasks are finished allow us to go to the beginning of the loop and
                // mutate GlobalState again
                thread_loop_end.notify_one();
            });
        }

        // wait until all threaded activities have finished
        thread_loop_end.notified().await;
    }

    fn process_command(&mut self, command: CommandData) -> anyhow::Result<()> {
        let global = &mut self.global_state;
        let bots = &mut self.bots;

        match command {
            CommandData::Mine(interfaces::Mine { sel }) => {
                let Selection2D { from, to } = sel.normalize();
                global.mine.mine(from, to, Some(MinePreference::FromDist));

                for bot in bots {
                    bot.actions.schedule(LazyStream::from(MineRegion));
                }
            }
            CommandData::GoTo(GoTo { location }) => {
                for bot in bots {
                    bot.actions
                        .schedule(BlockTravelTask::new(location, &bot.state));
                }
            }
            CommandData::Attack(Attack { name }) => {
                let player = self
                    .global_state
                    .players
                    .by_name(&name)
                    .context("player does not exist")?;
                let entity_id = self
                    .global_state
                    .entities
                    .by_player_uuid(player.uuid)
                    .context("could not find entity id for player")?;

                for bot in bots {
                    let task = LazyStream::from(AttackEntity::new(entity_id));
                    bot.actions.schedule(task);
                }
            }
            CommandData::Cancelled(_) | CommandData::Finished(_) => {}
        }

        Ok(())
    }
}
