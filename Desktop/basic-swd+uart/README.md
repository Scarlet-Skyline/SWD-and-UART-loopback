USB 调试器

引脚：
i. GPIO 模拟 SWD

    用外露的PWM信号脚：PA0（swdclk）  PA2（swdio）
    两者对应的MCU引脚：U14A 模块 TIM_CH1(PA0) TIM_CH3(PA2)

    板子上自带的swd：PA13（swdio） PA14（swdclk）
    MCU：U14A 模块上对应引脚

ii. UART回环测试

    选用的外露UART：USART1
    引脚：UART-RX PA10    UART-TX PA9
    MCU：U14A 模块上对应引脚

测试记录：

python3 -m serial.tools.list_ports

COM6

COM7

2 ports found

$\color{red}{\text{probe-rs list}}$

The following debug probes were found:

[0]: DM-MC02 CMSIS-DAP Demo -- c0de:cafc-0: (CMSIS-DAP)

$\color{red}{\text{python3 host.py COM6 ping}}$

No response. Check that this is the control interface.

//我们是不能肉眼分清哪个是uart，哪个是control port的，就用指令测试

//这个指令的输出，表示COM6其实是uart。COM7才是控制口。

$\color{red}{\text{python3 host.py COM7 ping}}$

OK DM-MC02

$\color{red}{\text{python3 host.py COM7 swd}}$

OK DPIDR 6BA02477

$\color{red}{\text{python3 host.py COM7 uart-loopback}}$

Expected b'HerKules UART\n', received b'ERR SYNTAX\n'

//我发了个错误，测试uart回环时没有调到COM6 uart口，导致

//COM7解析错误命令报错。不过也验证了分清不同接口的重要性

注意：由于同一根 USB-C 数据线也提供两个串口。UART 串口用于转发原始字节，
CAN/控制串口（control port）接收 PING、SWD ID 和 CAN 收发命令。

$\color{red}{\text{所以在测试输入指令时，随时切换COM口避免报错；而且在切换COM口的同时也要注意更换对应的硬件连接：}}$
前车之鉴

模拟SWD时，板子SWD的PA13和PA14通过杜邦线连接模拟的GPIO（我选的是PA0和PA2）
进行uart-loopback测试时，用USB-TTL线连接UART接口（我这里选的是少数外露的UART，是USART1）
并且在进行 host.py COM6 uart-loopback 测试时，先连接上USART1（短接对应的PA09和PA10），再输入命令
否则此指令可能不会执行

关于这两项配置的说明

1. 模拟调试器（swd）

COM7：control port（控制口）//ping、swd、CAN用这个通道
对应代码 main.rs 里的 control_port 函数和 protocol.rs。

怎么处理数据的：当你在 COM7 发送 PING，单片机不是直接把 PING 这几个字母发出去，而是用 Line 结构体把它拼成完整的字符串，然后去匹配。如果匹配到 PING，它就回复 OK DM-MC02；匹配到 SWD ID，它就去控制引脚跑 SWD 协议读 ID。

电脑和板子之间：靠一根 USB-C 数据线。
板子自己的引脚之间：靠你插的 杜邦线（PA0接PA14，PA2接PA13）

【电脑】---(1) USB线发送 "SWD ID" 命令---【芯片内部：USB外设】
【芯片内部：USB外设】---(2) 硬件中断通知 CPU---【芯片内部：CPU (主脑)】 
【芯片内部：CPU (主脑)】 ---(3) 运行 swd.rs 代码---【芯片引脚：PA0(时钟) / PA2(数据)】 
【芯片引脚：PA0(时钟) / PA2(数据)】 ---(4) 顺着你插的【杜邦线】流回去---【芯片引脚：PA14(SWCLK) / PA13(SWDIO)】
【芯片引脚：PA14(SWCLK) / PA13(SWDIO)】 ---(5) DAP硬件监听并回应---【芯片内部：DAP 调试模块】
【芯片内部：DAP 调试模块】---(6) 将 ID 数据吐出，顺着【杜邦线】原路返回---【芯片引脚：PA2】 ➔ 【CPU读取】 ➔ 【USB外设】
【芯片引脚：PA2】 ➔ 【CPU读取】 ➔ 【USB外设】 ---(7) 通过 USB 线传回电脑---【电脑打印结果：OK DPIDR ...】


2. uart（usart回环）

COM6：UART Port（串口/透传口）
COM6任务：电脑发出指令，它就原封不动地通过单片机真实的硬件串口（TX/RX引脚）发出去；硬件串口收到什么，它也就原封不动地传回电脑。
对应代码 main.rs 里的 uart_port 函数。
它是怎么处理数据的：没有命令解析，没有判断。只是：usb_in.read 收到数据 -> serial_out.write_all 从 TX 引脚发出去；serial_in.read 从 RX 引脚收到数据 -> usb_out.write_packet 发回电脑。（此串口常用于GPS/蓝牙通讯等）
当然，实际应用时想查看发回的数据：串口助手！！！

uart-loopback测试： host.py 里的这个测试，是为了验证 wiring.rs 里写的 UART 配置（PA9、PA10 引脚、115200 波特率）到底对不对。它模拟的是：电脑往 USB 发数据 -> USB 转给 UART 发送引脚 -> 通过导线回到 UART 接收引脚（因为短接了RX和TX） -> 再通过 USB 传回电脑。

$\color{red}{\text{所以，测试实操时，及时调整接线很重要}}$
