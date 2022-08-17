.PHONY: test

CXXFLAGS := -std=c++14 -Wall -Wfloat-conversion -Wmissing-declarations -fsanitize=float-divide-by-zero -g
CXX_INC := -I./ -I./include
CPP := clang++ $(CXXFLAGS) $(CXX_INC)

EXES := build/run_tests build/geojson2fgb

default: check_deps $(EXES)

test: build/run_tests
	# fails to find included files unless run from root
	(cd ../.. && src/cpp/build/run_tests -d yes)

clean:
	find . -name \*.o | xargs rm -f build/*

build/run_tests: packedrtree.o test/run_tests.o
	$(CPP) -o $@ $^

build/geojson2fgb: packedrtree.o examples/geojson2fgb.o
	$(CPP) -o $@ $^

check_deps: include/flatbuffers include/mapbox include/rapidjson

clean_deps:
	rm -r include/[^README]*

include/%:
	echo "Missing dependency: $@. Try (cd  ../.. && script/getcppdeps.sh)" && exit 1

%.o: %.cpp
	$(CPP) -c $< -o $@

