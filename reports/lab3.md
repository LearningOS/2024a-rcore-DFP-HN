# Lab3报告
## 实现的功能
+ 实现了进程的创建，分配独立的空间存放数据，拥有独立的页表，申请新的pid，在内存申请内核栈。如果文件名无效或者进程池满，则返回-1。


# 问答作业
实际是p2执行，stride采用8bit无符号整形存储，故stride的最大值为255，而p2执行一个时间片后，p2.stride = 250 + 10 = 260 % 256 = 4，下次找最小的stride依旧是p2。
当优先级为2时，pass最大为BIG_STRIDE/2，若STRIDE_MAX - STRIDE_MIN > BIG_STRIDE/2，则STRIDE_MAX在上一步执行前的最小可能取值为TRIDE_MAX - BIG_STRIDE/2 > STRIDE_MIN，这与之前的算法矛盾，STRIDE_MAX在上一步执行前并不是最小值，但却执行了。

```Rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // ...
        self.0.partial_cmp(&other.0)
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```

# 荣誉准则
1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

无

2. 此外，我也参考了以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

[rCore-Camp-Guide-2024A文档](https://learningos.cn/rCore-Camp-Guide-2024A/index.html)

[ChatGPT](https://chat.openai.com/chat)

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。