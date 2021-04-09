use fanotify::high_level::{Fanotify, FanotifyMode, FAN_OPEN, FAN_CLOSE, FAN_ACCESS, FAN_MODIFY, FAN_ONDIR, FanEvent, FAN_EVENT_ON_CHILD, FanotifyResponse, FAN_ACCESS_PERM, FAN_OPEN_PERM, FAN_MOVE, FAN_DELETE, Event, events_from_mask};
use nix::poll::{PollFd, PollFlags, poll};
use std::fmt;
use fanotify::low_level::{FAN_REPORT_FID, fanotify_init, FAN_CLOEXEC, O_CLOEXEC, O_RDONLY, FAN_CLASS_CONTENT, FAN_MARK_ADD, AT_FDCWD, fanotify_mark, fanotify_read, close_fd, FAN_CLASS_NOTIF, FAN_REPORT_DIR_FID};
use std::fs::read_link;

fn read_event(fd: i32) -> Vec<Event> {
    let mut result = Vec::new();
    let events = fanotify_read(fd);
    for metadata in events {
        let path = read_link(format!("/proc/self/fd/{}", metadata.fd)).unwrap_or_default();
        let path = path.to_str().unwrap();
        result.push(Event {
            fd: metadata.fd,
            path: String::from(path),
            events: events_from_mask(metadata.mask),
            pid: metadata.pid as u32,
        });
        close_fd(metadata.fd);
    }
    result
}

fn main() {
    let mode = FAN_OPEN | FAN_ACCESS | FAN_CLOSE | FAN_DELETE | FAN_MODIFY | FAN_ONDIR | FAN_EVENT_ON_CHILD;

    // let fd = Fanotify::new_with_blocking(FanotifyMode::CONTENT | FAN_REPORT_FID);
    let fd = fanotify_init(FAN_CLASS_NOTIF | FAN_REPORT_FID, O_CLOEXEC | O_RDONLY).unwrap();
    fanotify_mark(fd, FAN_MARK_ADD, mode, AT_FDCWD, "/home/jonjo/tmp").unwrap();

    let mut fds = [PollFd::new(fd, PollFlags::POLLIN)];
    loop {
        let poll_num = poll(&mut fds, -1).unwrap();
        if (poll_num > 0) {
            for event in read_event(fd) {
                for e in event.events {
                    // Ignore the Close and Move events as they're duplicated union types
                    if e != FanEvent::Close && e != FanEvent::Move {
                        println!("{} {:^15} {}", event.pid, format!("{:#?}", e), event.path)
                    }
                }
            }
        } else {
            eprintln!("poll_num <= 0!");
            break;
        }
    }
}
