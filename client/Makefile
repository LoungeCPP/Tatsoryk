ifeq "$(OS)" "Windows_NT"
	PATSEP := ;
else
	PATSEP := :
endif

NPM_BIN := $(shell npm bin)/
SOURCES := $(wildcard src/*.ts)
JS_OUT := $(patsubst src/%.ts,out/%.js,$(SOURCES))

export PATH := $(NPM_BIN)$(PATSEP)$(PATH)


.PHONY: all clean install

all: install tatsoryk.js

clean:
	rm -rf out/ tatsoryk.js

clean-install: clean
	rm -rf $(abspath $(NPM_BIN)..) typings/ bower_components/

install: $(NPM_BIN)tsc typings/browser.d.ts bower_components/ts-polyfills/index.js


tatsoryk.js: $(JS_OUT)
	browserify -o $@ $^
ifneq "$(CI)" ""
	yuglify --terminal --output $@ < $@
endif

$(NPM_BIN)tsc: package.json
	npm install

typings/browser.d.ts: typings.json
	typings install

bower_components/ts-polyfills/index.js: bower.json
	bower install


$(JS_OUT): $(SOURCES)
	tsc --newLine LF --noEmitOnError --outDir out/ --noImplicitAny --noImplicitReturns --pretty -t ES5 $^
