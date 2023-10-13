EXAMPLES ?= rdd rdd_static


CC = g++
CXXFLAGS += -Iincludes
LIBS += -Ltarget/release -lrust_device_detector 

.PHONY: all
all: $(EXAMPLES)

.PHONY: clean
clean:
	cargo clean
	rm -f $(EXAMPLES)

# Build the rust library example for c++
# LD_LIBRARY_PATH=target/release ./rdd
rdd: examples/rdd.cpp includes/rdd.h examples/rdd.cpp target/release/librust_device_detector.so
	$(CC) -o $@ examples/rdd.cpp $(CXXFLAGS) $(LIBS)

rdd_static: examples/rdd.cpp includes/rdd.h examples/rdd.cpp target/release/librust_device_detector.a
	$(CC) -o $@ examples/rdd.cpp target/release/librust_device_detector.a -Iincludes

includes/rdd.h target/release/librust_device_detector.so src/ffi.rs:
	cargo build --features full --release


