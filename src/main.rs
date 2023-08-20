mod db;
use db::{Criminal, Database};
use dotenv::dotenv;
use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::prelude::Message;
use serenity::prelude::*;

const HELP: &str = "Γεια σας! Αυτός είναι το Discord bot για τη διαχείριση εγκληματικών δεδομένων.

Διαθέσιμες εντολές:
- `!insert [όνομα] [αριθμό ταυτότητας]`: Εισάγετε ένα νέο εγκληματία στη βάση δεδομένων.
- `!add_felony [αριθμό ταυτότητας] [κατηγορία1, κατηγορία2, ...]`: Προσθέστε μια ή περισσότερες κατηγορίες εγκλημάτων σε έναν εγκληματία.
- `!find [αριθμό ταυτότητας]`: Αναζητήστε έναν εγκληματία με βάση το ID του.

Παράδειγμα χρήσης:
- `!insert Marixouana 12345`: Εισαγωγή νέου εγκληματία με όνομα \"Marixouana\" και αριθμό ταυτότητας \"12345\".
- `!add_felony 12345 Κλοπή, Απάτη`: Προσθήκη των κατηγοριών εγκλήματος \"Κλοπή\" και \"Απάτη\" στον εγκληματία με αριθμό ταυτότητας 12345.
- `!find 456`: Αναζήτηση του εγκληματία με αριθμό ταυτότητας 456.

Παρακαλώ να προσέχετε τη σωστή σύνταξη των εντολών και των ορισμάτων. Αντιδράστε σε κάθε εντολή για να λάβετε την αντίστοιχη απάντηση από το bot.

Καλή χρήση!";

#[group]
#[commands(find, insert, help, add_felony)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let db: Database = Database::new().await;
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = std::env::var("TOKEN").expect("No token no party (:");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = serenity::Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");
    {
        let mut data = client.data.write().await;
        data.insert::<Database>(db);
    }
    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn insert(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let db = data.get::<Database>().unwrap();
    let args = msg.content.split(" ").collect::<Vec<&str>>();
    if args.len() < 3 {
        msg.reply(ctx, "Wrong amount for arguments").await?;
        return Ok(());
    }

    let test = args[1..args.len() - 1].join(" ");
    let name = test.as_str();
    let at = args.last().unwrap(); //we know already taht args.len() == 3
    let test: Vec<_> = at.chars().filter(|x| !x.is_numeric()).collect();

    if test.len() > 0 {
        msg.reply(ctx, "Wrong arguments id must be numeric!")
            .await?;
        return Ok(());
    }

    let new_criminal = Criminal::new(name, at);

    let message = match db.insert_criminal(&new_criminal).await {
        Ok(_) => "Insertion complete",
        Err(_) => "Insertion failed",
    };

    msg.reply(ctx, message).await?;
    Ok(())
}

#[command]
async fn add_felony(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let db: &Database = data.get::<Database>().unwrap();
    let args = msg.content.split(" ").collect::<Vec<&str>>(); // TODO check args[1] if tis digits

    let mut felonies = msg
        .content
        .split(",")
        .filter(|x| !x.is_empty())
        .collect::<Vec<&str>>();

    felonies[0] = felonies[0]
        .split(" ")
        .collect::<Vec<&str>>()
        .iter()
        .last()
        .unwrap();

    if args.len() < 3 {
        msg.reply(ctx, "Wrong arguments!").await?;
        return Ok(());
    }
    let id = args.get(1).unwrap();

    let message: String = match db.get_criminal_by_id(id).await {
        Some(_) => "Adding felonies to criminal...".to_string(),
        None => format!("Could not find criminal with id: {}", id),
    };

    db.add_felony(id, &felonies).await.unwrap();
    msg.reply(ctx, message).await?;
    Ok(())
}

#[command]
async fn find(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let db: &Database = data.get::<Database>().unwrap();
    let args = msg.content.split(" ").collect::<Vec<&str>>();
    let id = args.get(1);
    if id == None {
        msg.reply(ctx, "Please provide an id").await?;
        return Ok(());
    }
    let result: Option<Criminal> = db.get_criminal_by_id(id.unwrap()).await;
    let output: String = match result {
        Some(c) => c.to_string(),
        None => String::from("Coould not find the criminal you asked ..."),
    };
    msg.reply(ctx, output).await?;
    Ok(())
}

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, HELP).await?;
    Ok(())
}
