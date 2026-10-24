#pragma once
// Portable replacements for the Grand Central Dispatch primitives the engine used:
//   dispatch_apply            -> me::parallelFor
//   serial queue + dispatch_after -> me::SerialQueue
//   dispatch timer source     -> me::RepeatingTimer
// Standard C++ only, so the engine builds on macOS, Windows and Linux.

#include <atomic>
#include <chrono>
#include <condition_variable>
#include <cstddef>
#include <functional>
#include <map>
#include <mutex>
#include <thread>
#include <vector>

namespace me {

/// Run fn(0) … fn(n-1) concurrently and return when all have finished
/// (same contract as dispatch_apply). fn(0) runs on the calling thread.
template <class F>
void parallelFor(size_t n, F&& fn) {
    if (n == 0) return;
    if (n == 1) { fn(size_t{0}); return; }
    std::vector<std::thread> threads;
    threads.reserve(n - 1);
    for (size_t i = 1; i < n; i++) {
        threads.emplace_back([&fn, i] { fn(i); });
    }
    fn(size_t{0});
    for (auto& t : threads) t.join();
}

/// One worker thread running tasks in order of their due time.
class SerialQueue {
public:
    using Clock = std::chrono::steady_clock;

    SerialQueue() : worker_([this] { run(); }) {}

    ~SerialQueue() {
        {
            std::lock_guard<std::mutex> lock(mutex_);
            stopping_ = true;
            tasks_.clear();
        }
        cv_.notify_all();
        if (worker_.joinable()) worker_.join();
    }

    SerialQueue(const SerialQueue&) = delete;
    SerialQueue& operator=(const SerialQueue&) = delete;

    void async(std::function<void()> fn) { asyncAt(Clock::now(), std::move(fn)); }

    void asyncAfter(double seconds, std::function<void()> fn) {
        auto delay = std::chrono::duration_cast<Clock::duration>(std::chrono::duration<double>(seconds));
        asyncAt(Clock::now() + delay, std::move(fn));
    }

    /// Drop tasks that have not started yet.
    void cancelPending() {
        std::lock_guard<std::mutex> lock(mutex_);
        tasks_.clear();
        idleCv_.notify_all();
    }

    /// Block until no task is running and none is due now (like dispatch_sync(queue, ^{})).
    void drain() {
        if (std::this_thread::get_id() == worker_.get_id()) return; // would deadlock
        std::unique_lock<std::mutex> lock(mutex_);
        idleCv_.wait(lock, [this] {
            return !running_ && (tasks_.empty() || tasks_.begin()->first > Clock::now());
        });
    }

    bool isWorkerThread() const { return std::this_thread::get_id() == worker_.get_id(); }

private:
    void asyncAt(Clock::time_point when, std::function<void()> fn) {
        {
            std::lock_guard<std::mutex> lock(mutex_);
            if (stopping_) return;
            tasks_.emplace(when, std::move(fn));
        }
        cv_.notify_all();
    }

    void run() {
        std::unique_lock<std::mutex> lock(mutex_);
        while (!stopping_) {
            if (tasks_.empty()) {
                idleCv_.notify_all();
                cv_.wait(lock);
                continue;
            }
            auto it = tasks_.begin();
            const Clock::time_point due = it->first; // copy: the node may be erased while we wait
            if (due > Clock::now()) {
                idleCv_.notify_all();
                cv_.wait_until(lock, due);
                continue;
            }
            auto fn = std::move(it->second);
            tasks_.erase(it);
            running_ = true;
            lock.unlock();
            fn();
            lock.lock();
            running_ = false;
            idleCv_.notify_all();
        }
    }

    std::mutex mutex_;
    std::condition_variable cv_;
    std::condition_variable idleCv_;
    std::multimap<Clock::time_point, std::function<void()>> tasks_;
    bool running_ = false;
    bool stopping_ = false;
    std::thread worker_; // declared last: starts after the members above exist
};

/// Calls fn every `interval` seconds on its own serial queue. setInterval() changes the
/// gap before the next call (it may be called from inside fn, like dispatch_source_set_timer).
class RepeatingTimer {
public:
    RepeatingTimer(double intervalSec, std::function<void()> fn)
        : interval_(intervalSec), fn_(std::move(fn)) {}

    ~RepeatingTimer() { stop(); }

    void start() {
        active_.store(true, std::memory_order_release);
        scheduleNext();
    }

    /// Stop and wait for an in-flight call to finish.
    void stop() {
        active_.store(false, std::memory_order_release);
        queue_.cancelPending();
        queue_.drain();
    }

    void setInterval(double intervalSec) {
        {
            std::lock_guard<std::mutex> lock(mutex_);
            interval_ = intervalSec;
        }
        // Outside the handler, restart the countdown with the new interval.
        if (!queue_.isWorkerThread() && active_.load(std::memory_order_acquire)) {
            queue_.cancelPending();
            scheduleNext();
        }
    }

private:
    void scheduleNext() {
        double interval;
        {
            std::lock_guard<std::mutex> lock(mutex_);
            interval = interval_;
        }
        queue_.asyncAfter(interval, [this] {
            if (!active_.load(std::memory_order_acquire)) return;
            fn_();
            if (active_.load(std::memory_order_acquire)) scheduleNext();
        });
    }

    std::mutex mutex_;
    double interval_;
    std::function<void()> fn_;
    std::atomic<bool> active_{false};
    SerialQueue queue_; // last: destroyed first, joining its thread before fn_ goes away
};

} // namespace me
