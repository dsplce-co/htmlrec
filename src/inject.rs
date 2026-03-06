// Overrides all time-based browser APIs to run on a virtual clock driven by
// __hrec_tick(time). Covers: performance.now, Date.now, rAF, setTimeout/setInterval,
// and CSS animations/transitions (tracked by birth time so currentTime is relative).
pub const INJECT: &str = r#"
    let currentTime = 0;
    let nextId = 0;

    const animationFrames = [];
    const timers = [];
    const animationBirths = new WeakMap();

    performance.now = () => currentTime;
    Date.now = () => currentTime | 0;

    window.requestAnimationFrame = callback => {
      const id = ++nextId;
      animationFrames.push({ id, callback });
      return id;
    };

    window.cancelAnimationFrame = id => {
      const index = animationFrames.findIndex(entry => entry.id === id);

      if (~index) {
        animationFrames.splice(index, 1);
      }
    };

    window.setTimeout = (callback, ms = 0, ...args) => {
      const id = ++nextId;
      timers.push({ id, at: currentTime + Math.max(0, +ms || 0), callback, args, loop: false });
      return id;
    };

    window.clearTimeout = id => {
      const index = timers.findIndex(entry => entry.id === id);

      if (~index) {
        timers.splice(index, 1);
      }
    };

    window.setInterval = (callback, ms = 0, ...args) => {
      const id = ++nextId;
      const delay = Math.max(1, +ms || 1);
      timers.push({ id, at: currentTime + delay, callback, args, loop: true, ms: delay });
      return id;
    };

    window.clearInterval = window.clearTimeout;

    function syncAnimations() {
      for (const animation of document.getAnimations()) {

        if (!animationBirths.has(animation)) {
          animationBirths.set(animation, currentTime);
          animation.pause();
        }

        animation.currentTime = currentTime - animationBirths.get(animation);
      }
    }

    window.__hrec_tick = async time => {
      currentTime = time;

      // drain timers in passes — each await lets promise continuations re-queue
      for (let pass = 0; pass < 100; pass++) {
        const due = timers.filter(entry => entry.at <= currentTime);

        if (!due.length) {
          break;
        }

        for (const entry of [...due]) {
          const index = timers.indexOf(entry);

          if (index < 0) {
            continue;
          }

          if (entry.loop) {
            timers[index].at += entry.ms;
          } else {
            timers.splice(index, 1);
          }

          try { entry.callback(...entry.args); } catch {}
        }

        await Promise.resolve();
      }

      const batch = animationFrames.splice(0);

      for (const { callback } of batch) {
        try { callback(currentTime); } catch {}
      }

      await Promise.resolve();
      syncAnimations();
    };
"#;
