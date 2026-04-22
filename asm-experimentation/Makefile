GAMES   := $(patsubst %/, %, $(wildcard */))
TARGETS := $(addsuffix .gb, $(GAMES))

ASM		:= rgbasm
LD		:= rgblink
FIX		:= rgbfix

all: $(TARGETS)

echo:
	echo $(GAMES)

define GAME_template
$(1).gb: $$(patsubst $(1)/%.asm, $(1)/%.o, $$(wildcard $(1)/*.asm))
	$$(LD) -o $$@ $$^
	$$(FIX) -v -p 0xFF $$@


$(1)/%.o: $(1)/%.asm
	$$(ASM) $$(ASFLAGS) -o $$@ $$<
endef

$(foreach game, $(GAMES), $(eval $(call GAME_template, $(game))))

fclean:
	rm -f $(TARGETS)
	echo $(wildcard hello-world/*.o)
	rm -f $(foreach game, $(GAMES), $(wildcard $(game)/*.o))

.PHONY: all fclean
