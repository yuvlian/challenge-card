use mihomo_rs::mihomo;
use mihomo_rs::Language;

pub struct UserData {
    pub name: String,
    pub signature: String,
    pub lv: u8,
    pub eq: u8,
    pub av: String,
}

pub async fn user(uid: u32) -> Result<UserData, Box<dyn std::error::Error>> {
    let sr_data = mihomo(uid, Language::EN).await?;
    let player_name = sr_data.player.name;
    let signature = sr_data.player.signature;
    let lv = sr_data.player.level;
    let eq = sr_data.player.world_level;
    let av = sr_data.player.avatar.icon;

    Ok(UserData {
        name: player_name,
        signature,
        lv,
        eq,
        av,
    })
}
