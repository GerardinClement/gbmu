while true; do                 
  if [ "$(find hello-world -name '*.asm' -newer hello-world.gb)" ]; then
    kill $(pgrep gameboy) 2>/dev/null; make hello-world.gb && (gameboy hello-world.gb &
  fi
  sleep 1
done

