use clap::{App, Arg, ArgMatches, SubCommand};

pub fn args() -> ArgMatches<'static> {
    let app = App::new("void")
        .version("1.0")
        .about("A http sink and recorder")
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(SubCommand::with_name("serve").about("Collect requests and abandon"))
        .subcommand(
            SubCommand::with_name("record")
                .about("Record requests")
                .arg(
                    Arg::with_name("compress")
                        .short("c")
                        .help("Compress with LZ4 while saving"),
                )
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .required(true)
                        .takes_value(true)
                        .help("Output directory"),
                )
                .arg(
                    Arg::with_name("threads")
                        .long("threads")
                        .short("t")
                        .help("How many recorder threads")
                        .required(false)
                        .takes_value(true)
                        .default_value("1"),
                ),
        );

    return app.get_matches();
}
