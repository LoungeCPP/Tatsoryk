ifeq "$(OS)" "Windows_NT"
    PATSEP := ;
else
    PATSEP := :
endif

NPM_BIN := $(shell npm bin)/
SOURCES := $(wildcard src/*.ts)
JS_OUT := $(patsubst src/%.ts,out/%.js,$(SOURCES))

export PATH := $(NPM_BIN)$(PATSEP)$(PATH)


.PHONY: all clean build install

all: install clean build

clean:
	rm -rf out/ tatsoryk.js

clean-install: clean
	rm -rf $(abspath $(NPM_BIN)..) typings

build: $(SOURCES)
	tsc --newLine LF --noEmitOnError --outDir out/ --noImplicitAny --noImplicitReturns --pretty -t ES5 $^
	browserify -o tatsoryk.js $(JS_OUT)

install: $(NPM_BIN)tsc typings/browser.d.ts


$(NPM_BIN)tsc: package.json
	npm install

typings/browser.d.ts: typings.json
	typings install
