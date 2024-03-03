CXX = g++
CC = gcc
FLEX = flex
BISON = bison
CFLAGS = -std=c11 -g -Wall
CXXFLAGS = -std=c++17 -g -Wall

FLEX_SRC = $(sort $(wildcard *.l))
BISON_SRC = $(sort $(wildcard *.y))
FLEX_CC_FILE = $(FLEX_SRC:.l=.yy.c)
BISON_CC_FILE = $(BISON_SRC:.y=.tab.c)
BISON_HEADER = $(BISON_CC_FILE:.c=.h)
FLEX_OBJ = $(FLEX_CC_FILE:.c=.o)
BISON_OBJ = $(BISON_CC_FILE:.c=.o)

calculator: $(FLEX_OBJ) $(BISON_OBJ)
	$(CC) -o $@ $^ -lfl -lm

$(BISON_OBJ): $(BISON_CC_FILE)
	$(CC) $(CFLAGS) -c $^ -o $@

$(BISON_CC_FILE) $(BISON_HEADER): $(BISON_SRC)
	$(BISON) -d -o $(BISON_CC_FILE) $^

$(FLEX_OBJ): $(FLEX_CC_FILE)
	$(CC) $(CFLAGS) -c $^ -o $@

$(FLEX_CC_FILE): $(FLEX_SRC) $(BISON_HEADER)
	$(FLEX) -o $@ $<

.PHONY: clean
clean:
	rm -rf $(FLEX_CC_FILE) $(BISON_CC_FILE) $(BISON_HEADER)
	rm -rf $(FLEX_OBJ) $(BISON_OBJ)
	rm -rf calculator