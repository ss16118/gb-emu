use clap::{Command, Arg};
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};

mod emulator;
use emulator::Emulator;


// Disable warnings for unused imports
#[allow(unused_variables, unused_mut, unused_imports)]
fn main() {
    /********************
    * Argument parsing
    *********************
    */
    let matches = Command::new("Game Boy Emulator")
        .version("0.1.0")
        .author("Siyuan Shen")
        .about("A Game Boy emulator written in Rust")
        .arg(Arg::new("rom_file")
                 .short('r')
                 .long("rom")
                 .required(true)
                 .num_args(1)
                 .help("Path to the ROM file"))
        .arg(Arg::new("log_file")
                 .short('l')
                 .long("log")
                 .num_args(1)
                 .default_value("gbemu.log")
                 .help("Path to the log file"))
        .arg(Arg::new("disable_logging")
                 .long("disable-logging")
                 .required(false)
                 .default_value("false")
                 .num_args(0)
                 .help("Enable logging"))
        .arg(Arg::new("enable_tracing")
                 .short('t')
                 .long("trace")
                 .required(false)
                 .default_value("false")
                 .num_args(0)
                 .help("Enable instruction tracing. Logs executed instructions to trace.log"))
        .arg(Arg::new("debug")
                 .short('d')
                 .long("debug")
                 .required(false)
                 .default_value("false")
                 .num_args(0)
                 .help("Enable debug mode. Logs CPU state after each instruction."))
        .get_matches();

    let rom_file = matches.get_one::<String>("rom_file").unwrap();
    let log_file = matches.get_one::<String>("log_file").unwrap();
    let disable_logging = matches.get_one::<bool>("disable_logging").unwrap();
    let enable_tracing = matches.get_one::<bool>("enable_tracing").unwrap();
    let debug = matches.get_one::<bool>("debug").unwrap();

    // Initialize the logger with the given log file
    // Implementation from:
    // https://medium.com/@nikmas_dev/advanced-logging-in-rust-with-log4rs-2d712bb322de
    if !disable_logging {
        let stdout = ConsoleAppender::builder().build();

        let log_file_appender = FileAppender::builder()
            .append(false)
            .encoder(Box::new(PatternEncoder::new(
                "{d(%Y-%m-%d %H:%M:%S)} | {({l}):5.5} | {m}{n}")))
            .build(log_file)
            .unwrap();

        // Trace file is 
        let trace_file_appender = FileAppender::builder()
            .append(false)
            .encoder(Box::new(PatternEncoder::new("{m}{n}")))
            .build("trace.log")
            .unwrap();
        
        let config: Config = Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .appender(Appender::builder().build("log_file", Box::new(log_file_appender)))
            .appender(Appender::builder().build("trace_file", Box::new(trace_file_appender)))
            .logger(log4rs::config::Logger::builder()
                    .appender("stdout")
                    .build("stdout", LevelFilter::Info))
            .logger(log4rs::config::Logger::builder()
                    .appender("trace_file")
                    .build("trace_file", LevelFilter::Trace))
            .build(Root::builder()
                    .appender("log_file")
                    .build(LevelFilter::Info))
            .unwrap();

        log4rs::init_config(config).unwrap();
        log::info!("Logging enabled [Log file: {}]", log_file);
        log::info!(target: "stdout", "Logger initialized");
        log::info!("Rom file: {}", rom_file);
        if *enable_tracing {
            log::info!("Tracing enabled [Trace file: trace.log]");
            log::warn!("This will slow down the emulator and produce a large log file.");
        }
    }
    // Initialize the emulator
    let mut emulator = Emulator::new(&rom_file, *enable_tracing);
        
    // Starts the emulator
    emulator.run(*debug);
}
