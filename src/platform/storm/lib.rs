#![crate_name = "platform"]
#![crate_type = "rlib"]
#![no_std]
#![feature(const_fn,lang_items)]

extern crate common;
extern crate drivers;
extern crate hil;
extern crate sam4l;
extern crate support;

use hil::Controller;
use hil::spi_master::SpiMaster;
use drivers::timer::AlarmToTimer;
use drivers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};

// HAL unit tests. To enable a particular unit test, uncomment
// the module here and uncomment the call to start the test in
// the init function below.
//mod gpio_dummy;
//mod spi_dummy;
//mod spi_driver;


#[allow(unused_variables,dead_code)]
pub struct DummyCB {
    val: u8
}
 
static mut spi_read_buf:  [u8; 64] = [0; 64];
static mut spi_write_buf: [u8; 64] = [0; 64];

#[allow(non_snake_case)]
pub struct Firestorm {
    chip: sam4l::chip::Sam4l,
    console: &'static drivers::console::Console<'static, sam4l::usart::USART>,
    gpio: drivers::gpio::GPIO<[&'static hil::gpio::GPIOPin; 13]>,
    timer: &'static drivers::timer::TimerDriver<'static, AlarmToTimer<'static,
                                VirtualMuxAlarm<'static, sam4l::ast::Ast>>>,
    tmp006: &'static drivers::tmp006::TMP006<'static, sam4l::i2c::I2CDevice, sam4l::gpio::GPIOPin>,
    spi: &'static drivers::spi::Spi<'static, sam4l::spi::Spi>,
    accelerometer: &'static drivers::FXOS8700CQ::FXOS8700CQ<'static, sam4l::i2c::I2CDevice>,
}

impl Firestorm {
    pub unsafe fn service_pending_interrupts(&mut self) {
        self.chip.service_pending_interrupts()
    }

    pub unsafe fn has_pending_interrupts(&mut self) -> bool {
        self.chip.has_pending_interrupts()
    }

    pub fn with_driver<F, R>(&mut self, driver_num: usize, f: F) -> R where
            F: FnOnce(Option<&hil::Driver>) -> R {

        match driver_num {
            0 => f(Some(self.console)),
            1 => f(Some(&self.gpio)),
            2 => f(Some(self.timer)),
            3 => f(Some(self.tmp006)),
            4 => f(Some(self.accelerometer)),
            5 => f(Some(self.spi)),
            _ => f(None)
        }
    }
}

macro_rules! static_init {
   ($V:ident : $T:ty = $e:expr) => {
        let $V : &mut $T = {
            // Waiting out for size_of to be available at compile-time to avoid
            // hardcoding an abitrary large size...
            static mut BUF : [u8; 1024] = [0; 1024];
            let mut tmp : &mut $T = mem::transmute(&mut BUF);
            *tmp = $e;
            tmp
        };
   }
}

pub unsafe fn init<'a>() -> &'a mut Firestorm {
    use core::mem;

    // Workaround for SB.02 hardware bug
    // TODO(alevy): Get rid of this when we think SB.02 are out of circulation
    sam4l::gpio::PA[14].enable();
    sam4l::gpio::PA[14].set();
    sam4l::gpio::PA[14].enable_output();

    static_init!(console : drivers::console::Console<sam4l::usart::USART> =
                    drivers::console::Console::new(&sam4l::usart::USART3,
                                       &mut drivers::console::WRITE_BUF));
    sam4l::usart::USART3.set_client(console);

    let ast = &sam4l::ast::AST;

    static_init!(mux_alarm : MuxAlarm<'static, sam4l::ast::Ast> =
                    MuxAlarm::new(&sam4l::ast::AST));
    ast.configure(mux_alarm);


    // the i2c address of the device is 0x40
    static_init!(tmp006 : drivers::tmp006::TMP006<'static, sam4l::i2c::I2CDevice, sam4l::gpio::GPIOPin> =
                    drivers::tmp006::TMP006::new(&sam4l::i2c::I2C2, 0x40, &sam4l::gpio::PA[9]));
    sam4l::gpio::PA[9].set_client(tmp006);

    static_init!(virtual_alarm1 : VirtualMuxAlarm<'static, sam4l::ast::Ast> =
                    VirtualMuxAlarm::new(mux_alarm));
    static_init!(vtimer1 : AlarmToTimer<'static,
                                VirtualMuxAlarm<'static, sam4l::ast::Ast>> =
                            AlarmToTimer::new(virtual_alarm1));
    virtual_alarm1.set_client(vtimer1);
    static_init!(timer : drivers::timer::TimerDriver<AlarmToTimer<'static,
                                VirtualMuxAlarm<'static, sam4l::ast::Ast>>> =
                            drivers::timer::TimerDriver::new(vtimer1));
    vtimer1.set_client(timer);

    // for accelerometer
    static_init!(accel_virtual_alarm : VirtualMuxAlarm<'static, sam4l::ast::Ast> =
                    VirtualMuxAlarm::new(mux_alarm));
    static_init!(accel_timer : AlarmToTimer<'static,
                                    VirtualMuxAlarm<'static, sam4l::ast::Ast>> =
                            AlarmToTimer::new(accel_virtual_alarm));
    accel_virtual_alarm.set_client(accel_timer);
    static_init!(accelerometer : drivers::FXOS8700CQ::FXOS8700CQ<'static,
                                    sam4l::i2c::I2CDevice> =
        drivers::FXOS8700CQ::FXOS8700CQ::new(&sam4l::i2c::I2C2, accel_timer));
    accel_timer.set_client(accelerometer);

    // Configure SPI pins: CLK, MISO, MOSI, CS3
    sam4l::gpio::PC[ 6].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PC[ 4].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PC[ 5].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PC[ 1].configure(Some(sam4l::gpio::PeripheralFunction::A));
    // Initialize and enable SPI HAL
    static_init!(spi: drivers::spi::Spi<'static, sam4l::spi::Spi> =
                      drivers::spi::Spi::new(&mut sam4l::spi::SPI));
    spi.config_buffers(&mut spi_read_buf, &mut spi_write_buf);

    static_init!(firestorm : Firestorm = Firestorm {
        chip: sam4l::chip::Sam4l::new(),
        console: console,
        gpio: drivers::gpio::GPIO::new(
            [ &sam4l::gpio::PC[10], &sam4l::gpio::PC[19]
            , &sam4l::gpio::PC[13], &sam4l::gpio::PA[17]
            , &sam4l::gpio::PC[20], &sam4l::gpio::PA[19]
            , &sam4l::gpio::PA[14], &sam4l::gpio::PA[16]
            , &sam4l::gpio::PA[13], &sam4l::gpio::PA[11]
            , &sam4l::gpio::PA[10], &sam4l::gpio::PA[12]
            , &sam4l::gpio::PC[09]]),
        timer: timer,
        tmp006: tmp006,
        spi: spi,
        accelerometer: accelerometer,
    });

    sam4l::usart::USART3.configure(sam4l::usart::USARTParams {
        //client: &console,
        baud_rate: 115200,
        data_bits: 8,
        parity: hil::uart::Parity::None
    });

    sam4l::gpio::PB[09].configure(Some(sam4l::gpio::PeripheralFunction::A));
    sam4l::gpio::PB[10].configure(Some(sam4l::gpio::PeripheralFunction::A));

    // Configure I2C SDA and SCL pins
    sam4l::gpio::PA[21].configure(Some(sam4l::gpio::PeripheralFunction::E));
    sam4l::gpio::PA[22].configure(Some(sam4l::gpio::PeripheralFunction::E));

    // Uncommenting the following line will cause the device to use the 
    // SPI HAL to write [8, 7, 6, 5, 4, 3, 2, 1] once over the SPI then 
    // echo the 8 bytes read from the slave continuously. 
    //spi_dummy::spi_dummy_test();

    // Uncommenting the following line will toggle the LED whenever the value of
    // Firestorm's pin 8 changes value (e.g., connect a push button to pin 8 and
    // press toggle it).
    //gpio_dummy::gpio_dummy_test();

    sam4l::spi::SPI.set_active_peripheral(sam4l::spi::Peripheral::Peripheral1);
    sam4l::spi::SPI.init(spi as &hil::spi_master::SpiCallback);
    sam4l::spi::SPI.enable();

    firestorm.console.initialize();
    firestorm
}

use core::fmt::Arguments;

#[cfg(not(test))]
#[lang="panic_fmt"]
#[no_mangle]
pub unsafe extern fn rust_begin_unwind(_args: &Arguments,
    _file: &'static str, _line: usize) -> ! {
    use hil::uart::UART;
    use core::fmt::*;
    use support::nop;

    sam4l::usart::USART3.configure(sam4l::usart::USARTParams {
        baud_rate: 115200,
        data_bits: 8,
        parity: hil::uart::Parity::None
    });
    sam4l::usart::USART3.enable_tx();

    struct Writer;

    impl Write for Writer {
        fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
            unsafe {
                for c in s.bytes() {
                    sam4l::usart::USART3.send_byte(c);
                }
            }
            Ok(())
        }
    }

    let _ = Writer.write_fmt(format_args!("Kernel panic... Sorry!\r\n"));

    let led = &sam4l::gpio::PC[10];
    led.enable_output();
    loop {
        for _ in 0..1000000 {
            led.set();
            nop();
        }
        for _ in 0..1000000 {
            led.clear();
            nop();
        }
    }
}

