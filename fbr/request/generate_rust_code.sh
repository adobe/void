cd $(dirname ${0})
printf "Generating flatbuffer code. CWD ="
pwd
flatc --version
flatc --rust -o ../../src request.fbs
