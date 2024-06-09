use chrono::DateTime;
use whrd::Session;
use crate::config::GlobalConfig;

pub fn print_summary(mut sessions: Vec<Session>, config: GlobalConfig) {
    fn max_key_with_min<T, F>(sessions: &[Session], get_key: F, floor: T) -> T
        where
            T: Ord + Default,
            F: Fn(&Session) -> T
    {
        sessions.iter()
            .max_by_key(|s| get_key(s))
            .map(get_key)
            .unwrap_or_default()
            .max(floor)
    }


    sessions.sort_unstable_by_key(|s| s.login_time);
    sessions.sort_by_key(|s| !s.active); // We want active first

    const ACTIVE_PADDING: usize = 2;
    let host_padding = max_key_with_min(&sessions, |s| s.host.as_deref().map_or(0, |str| str.len()), 5);
    let remote_padding = max_key_with_min(&sessions, |s| s.remote.as_deref().map_or(0, |str| str.len()), 7);
    let username_padding = max_key_with_min(&sessions, |s| s.user.len(), 5);
    let tty_padding = max_key_with_min(&sessions, |s| s.tty.len(), 4);
    let pid_padding = max_key_with_min(&sessions, |s| s.pid.abs().checked_ilog10().unwrap_or_default() + 1 + (s.pid < 0) as u32, 4);

    if config.include_inactive {
        println!("{:pad_0$}  {:<pad_1$}  {:<pad_2$}  {:<pad_3$}  {:<pad_4$}  {:<pad_5$}  Since",
                 "Act",
                 "Host",
                 "Source",
                 "User",
                 "TTY",
                 "PID",
                 pad_0 = ACTIVE_PADDING,
                 pad_1 = host_padding,
                 pad_2 = remote_padding,
                 pad_3 = username_padding,
                 pad_4 = tty_padding,
                 pad_5 = pid_padding as usize);
    } else {
        println!("{:<pad_1$}  {:<pad_2$}  {:<pad_3$}  {:<pad_4$}  {:<pad_5$}  Since",
                 "Host",
                 "Source",
                 "User",
                 "TTY",
                 "PID",
                 pad_1 = host_padding,
                 pad_2 = remote_padding,
                 pad_3 = username_padding,
                 pad_4 = tty_padding,
                 pad_5 = pid_padding as usize);
    }

    for session in sessions {
        if !config.include_inactive && !session.active {
            continue;
        }

        let active = if session.active {
            '*'
        } else {
            ' '
        };

        let host = session.host.unwrap_or_else(|| ' '.to_string());
        let remote = session.remote.unwrap_or_else(|| config.source.clone());

        let datetime = DateTime::from_timestamp(session.login_time, 0).unwrap();
        let time = datetime.format("%Y-%m-%d %H:%M:%S");

        if config.include_inactive {
            println!(" {:<pad_0$}  {:<pad_1$}  {:<pad_2$}  {:<pad_3$}  {:<pad_4$}  {:<pad_5$}  {}",
                     active,
                     host,
                     remote,
                     session.user,
                     session.tty,
                     session.pid,
                     time,
                     pad_0 = ACTIVE_PADDING,
                     pad_1 = host_padding,
                     pad_2 = remote_padding,
                     pad_3 = username_padding,
                     pad_4 = tty_padding,
                     pad_5 = pid_padding as usize);
        } else {
            println!("{:<pad_1$}  {:<pad_2$}  {:<pad_3$}  {:<pad_4$}  {:<pad_5$}  {}",
                     host,
                     remote,
                     session.user,
                     session.tty,
                     session.pid,
                     time,
                     pad_1 = host_padding,
                     pad_2 = remote_padding,
                     pad_3 = username_padding,
                     pad_4 = tty_padding,
                     pad_5 = pid_padding as usize);
        }
    }
}
