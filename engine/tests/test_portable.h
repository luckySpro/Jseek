#pragma once
// Part 77: portability layer (M1) — Unicode lowercasing and GCD replacements.
#include "Core/Concurrency.h"
#include "Core/StringUtils.h"
#include <atomic>
#include <chrono>
#include <mutex>
#include <thread>
#include <vector>

static void runPortableTests() {
    std::cout << "── Part 77: Portability layer ──\n";
    using me::toLower;
    check(toLower("README.TXT") == "readme.txt", "77.1 ASCII lowercase");
    check(toLower("ÉCOLE Ärger ÖL ÜBER") == "école ärger öl über", "77.2 Latin-1 lowercase");
    check(toLower("ŁÓDŹ ŻÓŁW") == "łódź żółw", "77.3 Latin Extended-A (Polish)");
    check(toLower("ĞÜŞİÖÇ") == "ğüşiöç", "77.4 Turkish incl. dotted I");
    check(toLower("ΑΘΗΝΑ Ώρα") == "αθηνα ώρα", "77.5 Greek");
    check(toLower("МОСКВА ЁЖ") == "москва ёж", "77.6 Cyrillic");
    check(toLower("ＡＢＣ全角") == "ａｂｃ全角", "77.7 Fullwidth Latin, CJK unchanged");
    check(toLower("中文文件名.PDF") == "中文文件名.pdf", "77.8 Chinese name, ASCII extension lowered");
    check(toLower("日本語ファイル.TXT") == "日本語ファイル.txt", "77.9 Japanese name");
    check(toLower("Tiếng VIỆT") == "tiếng việt", "77.10 Vietnamese (Latin Extended Additional)");
    check(toLower("ȴȵȶȷ") == "ȴȵȶȷ", "77.11 lowercase-only letters unchanged");
    check(toLower(std::string("bad\xff\xfe" "X")) == std::string("bad\xff\xfe" "x"), "77.12 invalid UTF-8 passes through");

    std::atomic<int> hits[64];
    for (auto& h : hits) h = 0;
    me::parallelFor(64, [&](size_t i) { hits[i]++; });
    bool once = true;
    for (auto& h : hits) once = once && h == 1;
    check(once, "77.13 parallelFor runs every index exactly once");

    {
        me::SerialQueue q;
        std::vector<int> order;
        std::mutex m;
        q.asyncAfter(0.15, [&] { std::lock_guard<std::mutex> l(m); order.push_back(2); });
        q.async([&] { std::lock_guard<std::mutex> l(m); order.push_back(1); });
        std::this_thread::sleep_for(std::chrono::milliseconds(300));
        q.drain();
        check(order == std::vector<int>{1, 2}, "77.14 SerialQueue runs tasks in due-time order");
        std::atomic<bool> ran{false};
        q.asyncAfter(0.2, [&] { ran = true; });
        q.cancelPending();
        std::this_thread::sleep_for(std::chrono::milliseconds(300));
        check(!ran, "77.15 SerialQueue::cancelPending drops delayed task");
    }
    {
        std::atomic<int> n{0};
        me::RepeatingTimer* tp = nullptr;
        me::RepeatingTimer t(0.05, [&] { n++; if (n == 2) tp->setInterval(0.02); });
        tp = &t;
        t.start();
        std::this_thread::sleep_for(std::chrono::milliseconds(400));
        t.stop();
        int after = n;
        std::this_thread::sleep_for(std::chrono::milliseconds(150));
        check(n >= 5, "77.16 RepeatingTimer fires repeatedly (interval changed in handler)");
        check(n == after, "77.17 RepeatingTimer::stop prevents further fires");
    }
}
