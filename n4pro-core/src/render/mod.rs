// 渲染模块入口

pub mod cache;
pub mod canvas;
pub mod key;
pub mod knob;
pub mod text;
pub mod touchbar;

pub use cache::ImageCache;
pub use canvas::Canvas;

use std::sync::Mutex;

/// 并行渲染：把多个渲染任务交给固定 2 个工作线程执行，按原顺序返回结果
/// 仅用于整页刷新的瞬时全速阶段；空闲路径不经过此函数
pub fn render_parallel<'a, R: Send>(
    jobs: Vec<(usize, Box<dyn FnOnce() -> R + Send + 'a>)>,
) -> Vec<(usize, R)> {
    if jobs.is_empty() {
        return Vec::new();
    }
    if jobs.len() == 1 {
        let (idx, f) = jobs.into_iter().next().expect("jobs 非空");
        return vec![(idx, f())];
    }
    let queue = Mutex::new(jobs);
    let results = Mutex::new(Vec::new());
    let workers = 2usize.min(queue.lock().unwrap_or_else(|e| e.into_inner()).len());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let next = queue
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .pop();
                let Some((idx, f)) = next else { break };
                let r = f();
                results
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push((idx, r));
            });
        }
    });
    let mut out = results.into_inner().unwrap_or_default();
    out.sort_by_key(|(i, _)| *i);
    out
}
