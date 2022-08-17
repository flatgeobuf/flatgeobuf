#include <format>
#include <fstream>
#include <iostream>
#include <sstream>
#include <unistd.h>

#include "flatbuffers/flatbuffers.h"
#include "header_generated.h"
//#include "feature_generated.h"

#include "../geojson.h"

#define DEBUGFLAG

void debug_print() { std::cerr << std::endl; }
template <typename Head, typename... Tail> void debug_print(Head H, Tail... T)
{
  std::cerr << ' ' << H;
  debug_print(T...);
}

#ifdef DEBUGFLAG
#define DEBUG(...)                                                             \
  std::cerr << "dbg(" << #__VA_ARGS__ << "):", debug_print(__VA_ARGS__)
#else
#define DEBUG(...)                                                             \
  do {                                                                         \
  } while (0)
#endif

using namespace mapbox::geojson;
using namespace flatbuffers;
using namespace FlatGeobuf;

const std::string getFixture(const std::string &path)
{
  std::ifstream t(path);
  std::stringstream buffer;
  buffer << t.rdbuf();
  return buffer.str();
}

int main(int argc, char *argv[])
{
  if (argc != 2) {
    std::fprintf(
        stderr,
        "Wrong args.\nUsage:\n    %s <filename.geojson> > <output.fgb>\n",
        argv[0]);
    exit(1);
  }
  char *inputFilename = argv[1];
  DEBUG(inputFilename);

  if (isatty(1)) {
    std::fprintf(stderr,
                 "This program prints to stdout. You must redirect output to "
                 "file.\nUsage:\n    %s <filename.geojson> > <output.fgb>\n",
                 argv[0]);
    exit(1);
  }

  auto fixtureString = getFixture(inputFilename);
  DEBUG(fixtureString);

  // TODO: handle other valid top level geometries, currently all the
  // fixtures are feature_collection
  auto geojson = parse(fixtureString).get<feature_collection>();

  std::vector<uint8_t> flatgeobuf;
  bool createIndex = true;
  serialize(flatgeobuf, geojson, createIndex);
  std::cout.write((char *)&flatgeobuf[0], flatgeobuf.size());
}
