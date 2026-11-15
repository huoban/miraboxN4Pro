// 动作线程池：常驻 worker + 空闲回收
// 规则：并发上限 15；至少保留 1 个常驻；空闲超过 3 个时多余线程退出；单线程空闲 10 秒回收
// 仅用于动作执行，不承载 HID 读/写线程（那两条线程保持常驻独立）

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct State {
    queue: VecDeque<Job>,
    workers: usize,
    idle: usize,
}

pub struct ActionPool {
    state: Arc<(Mutex<State>, Condvar)>,
    max: usize,
    max_idle: usize,
}

impl ActionPool {
    /// 创建线程池，启动 1 个常驻 worker
    pub fn new(max: usize, max_idle: usize) -> Self {
        let pool = Self {
            state: Arc::new((
                Mutex::new(State {
                    queue: VecDeque::new(),
                    workers: 0,
                    idle: 0,
                }),
                Condvar::new(),
            )),
            max,
            max_idle,
        };
        pool.spawn_worker();
        pool
    }

    /// 提交任务：队列积压且未达上限时扩容，否则唤醒空闲 worker
    pub fn submit<F: FnOnce() + Send + 'static>(&self, job: F) {
        let (lock, cv) = &*self.state;
        let mut s = lock.lock().unwrap_or_else(|e| e.into_inner());
        s.queue.push_back(Box::new(job));
        if s.queue.len() > s.idle && s.workers < self.max {
            s.workers += 1;
            drop(s);
            self.spawn_worker();
            cv.notify_all();
        } else {
            drop(s);
            cv.notify_one();
        }
    }

    /// 启动一个 worker；空闲超限或超过 24 小时自动退出
    fn spawn_worker(&self) {
        let state = Arc::clone(&self.state);
        let max_idle = self.max_idle;
        thread::spawn(move || {
            let (lock, cv) = &*state;
            loop {
                let job = {
                    let mut s = lock.lock().unwrap_or_else(|e| e.into_inner());
                    s.idle += 1;
                    let idle_since = Instant::now();
                    loop {
                        if let Some(j) = s.queue.pop_front() {
                            s.idle -= 1;
                            break Some(j);
                        }
                        let (g, t) = cv
                            .wait_timeout(s, Duration::from_secs(2))
                            .unwrap_or_else(|e| e.into_inner());
                        s = g;
                        if t.timed_out() && s.queue.is_empty() {
                            let idle_long = idle_since.elapsed() >= Duration::from_secs(10);
                            if (s.idle > max_idle || idle_long) && s.idle > 1 {
                                s.idle -= 1;
                                s.workers -= 1;
                                return;
                            }
                        }
                    }
                };
                if let Some(j) = job {
                    j();
                }
            }
        });
    }
}
