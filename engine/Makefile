# JSeek search engine (C++). Imported from JMacEverything (MIT, originally joshua-wu/MacEverything).
CXX ?= clang++
CXXFLAGS = -std=c++20 -O2 -Wall -Wextra
HOMEBREW_PREFIX ?= /opt/homebrew
RE2_CFLAGS = -I$(HOMEBREW_PREFIX)/opt/re2/include -I$(HOMEBREW_PREFIX)/include
RE2_LDFLAGS = -L$(HOMEBREW_PREFIX)/opt/re2/lib -L$(HOMEBREW_PREFIX)/lib -lre2
FRAMEWORKS = -framework CoreServices
CORE_SRCS = $(wildcard Core/*.cpp)

.PHONY: test-fast clean
test_all: test_all.cpp $(CORE_SRCS)
	$(CXX) $(CXXFLAGS) $(RE2_CFLAGS) $(FRAMEWORKS) $(RE2_LDFLAGS) -I. -ICore $^ -o $@

test-fast: test_all
	./test_all --fast

clean:
	rm -f test_all
