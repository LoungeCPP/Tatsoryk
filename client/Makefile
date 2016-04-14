ifeq "$(OS)" "Windows_NT"
	PATSEP := ;
else
	PATSEP := :
endif

ifneq "$(CI)" ""
	VENDOR_LIB_PART := $$4
else
	VENDOR_LIB_PART := $$4
endif

NPM_BIN := $(shell npm bin)/
SOURCES := $(wildcard src/*.ts)
JS_OUT := $(patsubst src/%.ts,out/%.js,$(SOURCES))

export PATH := $(NPM_BIN)$(PATSEP)$(PATH)

VENDOR_LIBS := $(shell awk '{print($$1)}' vendor.libs)


.PHONY: all clean install

all: install tatsoryk.js index.html

clean:
	rm -rf out/ tatsoryk.js

clean-install: clean
	rm -rf index.html $(abspath $(NPM_BIN)..) typings vendor

install: $(NPM_BIN)tsc typings/browser.d.ts


index.html: index.html.template $(patsubst %,vendor/%.js,$(VENDOR_LIBS))
	sed $(foreach vlib,$(subst $<,,$^),-e 's:<script src="tatsoryk.js"></script>:<script src="$(vlib)"></script>\n        &:') $< > $@

tatsoryk.js: $(JS_OUT)
	browserify -o $@ $^
ifneq "$(CI)" ""
	yuglify --terminal --output $@ < $@
endif

$(NPM_BIN)tsc: package.json
	npm install

typings/browser.d.ts: typings.json
	typings install


$(JS_OUT): $(SOURCES)
	tsc --newLine LF --noEmitOnError --outDir out/ --noImplicitAny --noImplicitReturns --pretty -t ES5 $^

vendor/%.js: vendor.libs
	@mkdir -p $(dir $@)
	curl -SsLo $@ $(shell awk '/$(patsubst vendor/%.js,%,$@)/ {print($$2 "/" $(VENDOR_LIB_PART))}' $^)
