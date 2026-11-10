(function () {
  function animateCounter(el) {
    var rawTarget = el.getAttribute("data-counter");
    if (!rawTarget) return;
    var target = parseFloat(rawTarget);
    if (isNaN(target)) return;

    var prefix = el.getAttribute("data-prefix") || "";
    var suffix = el.getAttribute("data-suffix") || "";
    var isFloat = rawTarget.indexOf(".") !== -1;
    var decimals = isFloat ? (rawTarget.split(".")[1] || "").length : 0;
    var duration = 1200;
    var start = performance.now();

    function update(now) {
      var elapsed = Math.min((now - start) / duration, 1);
      var ease = 1 - Math.pow(1 - elapsed, 3);
      var current = target * ease;
      el.textContent = prefix + (isFloat ? current.toFixed(decimals) : Math.round(current).toLocaleString()) + suffix;
      if (elapsed < 1) {
        requestAnimationFrame(update);
      } else {
        el.textContent = prefix + (isFloat ? target.toFixed(decimals) : target.toLocaleString()) + suffix;
      }
    }
    requestAnimationFrame(update);
  }

  function initCounters() {
    var elements = document.querySelectorAll("[data-counter]");
    if (!elements.length) return;

    if (window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      elements.forEach(function (el) {
        var rawTarget = el.getAttribute("data-counter");
        var prefix = el.getAttribute("data-prefix") || "";
        var suffix = el.getAttribute("data-suffix") || "";
        el.textContent = prefix + rawTarget + suffix;
      });
      return;
    }

    if ("IntersectionObserver" in window) {
      var observer = new IntersectionObserver(
        function (entries, obs) {
          entries.forEach(function (entry) {
            if (entry.isIntersecting) {
              animateCounter(entry.target);
              obs.unobserve(entry.target);
            }
          });
        },
        { threshold: 0.15 }
      );

      elements.forEach(function (el) {
        observer.observe(el);
      });
    } else {
      elements.forEach(animateCounter);
    }
  }

  function initClipboard() {
    var buttons = document.querySelectorAll("[data-copy]");
    var legacyCopy = function (text) {
      var ta = document.createElement("textarea");
      ta.value = text;
      ta.setAttribute("readonly", "");
      ta.style.position = "fixed";
      ta.style.opacity = "0";
      document.body.appendChild(ta);
      ta.select();
      try {
        document.execCommand("copy");
      } catch (e) {}
      document.body.removeChild(ta);
    };
    buttons.forEach(function (btn) {
      btn.addEventListener("click", function () {
        var text = btn.getAttribute("data-copy");
        if (!text) return;
        var done = function () {
          var original = btn.textContent;
          btn.textContent = "Copied!";
          btn.classList.add("copied");
          setTimeout(function () {
            btn.textContent = original;
            btn.classList.remove("copied");
          }, 2000);
        };
        if (navigator.clipboard && navigator.clipboard.writeText) {
          navigator.clipboard.writeText(text).then(done, function () {
            legacyCopy(text);
            done();
          });
        } else {
          legacyCopy(text);
          done();
        }
      });
    });
  }

  function initSubsystemTabs() {
    var containers = document.querySelectorAll(".subsystem-container");
    if (!containers.length) return;

    containers.forEach(function (container) {
      var tabs = Array.prototype.slice.call(container.querySelectorAll(".subsystem-tab"));
      var panes = container.querySelectorAll(".subsystem-pane");

      function selectTab(tab, focus) {
        tabs.forEach(function (t) {
          var active = t === tab;
          t.classList.toggle("active", active);
          t.setAttribute("aria-selected", active ? "true" : "false");
          t.tabIndex = active ? 0 : -1;
        });
        panes.forEach(function (p) {
          var isActive = p.id === tab.getAttribute("data-target");
          p.classList.toggle("active", isActive);
          if (p.hasAttribute("hidden") || isActive) {
            p.hidden = !isActive;
          }
        });
        if (focus) tab.focus();
      }

      tabs.forEach(function (tab, idx) {
        tab.addEventListener("click", function () {
          selectTab(tab, false);
        });
        tab.addEventListener("keydown", function (event) {
          var delta = 0;
          if (event.key === "ArrowRight" || event.key === "ArrowDown") delta = 1;
          if (event.key === "ArrowLeft" || event.key === "ArrowUp") delta = -1;
          if (event.key === "Home") delta = -idx;
          if (event.key === "End") delta = tabs.length - 1 - idx;
          if (!delta) return;
          event.preventDefault();
          selectTab(tabs[(idx + delta + tabs.length) % tabs.length], true);
        });
      });
    });
  }



  function initBootRace() {
    var container = document.querySelector(".boot-race");
    if (!container) return;

    var rows = container.querySelectorAll(".race-row");
    var result = container.querySelector("#raceResult");
    var replay = container.querySelector("#bootRaceReplay");
    if (!rows.length || !result) return;

    var frameId = null;

    function run() {
      if (frameId) cancelAnimationFrame(frameId);

      var maxTime = 0;
      rows.forEach(function (row) {
        var t = parseFloat(row.getAttribute("data-time"));
        if (!isNaN(t) && t > maxTime) maxTime = t;
      });
      if (!maxTime) return;

            var scale = 3500 / maxTime;
      var start = performance.now();

      rows.forEach(function (row) {
        row.querySelector(".race-fill").style.width = "0%";
      });
      result.textContent = "running...";

      function frame(now) {
        var elapsed = now - start;
        var allDone = true;

        rows.forEach(function (row) {
          var t = parseFloat(row.getAttribute("data-time"));
          if (isNaN(t)) return;
          var progress = Math.min(elapsed / (t * scale), 1);
          row.querySelector(".race-fill").style.width = (progress * 100).toFixed(1) + "%";
          if (progress < 1) allDone = false;
        });

        if (allDone) {
          frameId = null;
          var pomme = parseFloat(rows[rows.length - 1].getAttribute("data-time"));
          var slowest = parseFloat(rows[0].getAttribute("data-time"));
          if (!isNaN(pomme) && !isNaN(slowest)) {
            result.textContent = "Pomme wins: interactive " + (slowest - pomme).toFixed(1) + "s earlier";
          }
          return;
        }
        frameId = requestAnimationFrame(frame);
      }

      frameId = requestAnimationFrame(frame);
    }

    if (replay) replay.addEventListener("click", run);

    if (window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      rows.forEach(function (row) {
        row.querySelector(".race-fill").style.width = "100%";
      });
      result.textContent = "Pomme 1.4s / Vanilla 11.8s / Lunar 14.5s";
      return;
    }

    if ("IntersectionObserver" in window) {
      var obs = new IntersectionObserver(
        function (entries, o) {
          entries.forEach(function (entry) {
            if (entry.isIntersecting) {
              run();
              o.unobserve(entry.target);
            }
          });
        },
        { threshold: 0.3 }
      );
      obs.observe(container);
    } else {
      run();
    }
  }

  function initTrackerFilter() {
    var searchInput = document.getElementById("trackerSearch");
    var categoryPills = document.querySelectorAll(".cat-pill");
    var statusPills = document.querySelectorAll(".status-pill");
    var items = document.querySelectorAll(".tracker-item");
    var groups = document.querySelectorAll(".tracker-group");
    var counterEl = document.getElementById("trackerCount");

    if (!items.length) return;

    var originals = new Map();
    items.forEach(function (item) {
      var h4 = item.querySelector("h4");
      var p = item.querySelector(".tracker-info p");
      var crate = item.querySelector(".tracker-crate");
      if (h4 && p) {
        originals.set(item, {
          h4: h4,
          name: h4.textContent,
          p: p,
          desc: p.textContent,
          crateText: crate ? crate.textContent : ""
        });
      }
    });

    var currentCategory = "all";
    var currentStatus = "all";
    var currentSearch = "";

    function highlight(entry, query) {
      if (entry.name.toLowerCase().indexOf(query) !== -1) {
        writeMarked(entry.h4, entry.name, query);
        entry.p.textContent = entry.desc;
        return;
      }
      entry.h4.textContent = entry.name;
      writeMarked(entry.p, entry.desc, query);
    }

    function writeMarked(el, text, query) {
      var idx = text.toLowerCase().indexOf(query);
      if (idx === -1) {
        el.textContent = text;
        return;
      }
      el.textContent = "";
      el.appendChild(document.createTextNode(text.slice(0, idx)));
      var mark = document.createElement("mark");
      mark.textContent = text.slice(idx, idx + query.length);
      el.appendChild(mark);
      el.appendChild(document.createTextNode(text.slice(idx + query.length)));
    }

    function applyFilter() {
      var visibleCount = 0;
      items.forEach(function (item) {
        var cat = item.getAttribute("data-category") || "";
        var status = item.getAttribute("data-status") || "";
        var text = (item.textContent || "").toLowerCase();

        var matchCat = currentCategory === "all" || cat === currentCategory;
        var matchStatus = currentStatus === "all" || status === currentStatus;
        var matchSearch = !currentSearch || text.indexOf(currentSearch) !== -1;

        if (matchCat && matchStatus && matchSearch) {
          var entry = originals.get(item);
          if (entry && currentSearch) {
            highlight(entry, currentSearch);
          } else if (entry) {
            entry.h4.textContent = entry.name;
            entry.p.textContent = entry.desc;
          }
          item.style.display = "flex";
          visibleCount++;
        } else {
          item.style.display = "none";
        }
      });

      groups.forEach(function (group) {
        var hasVisible = false;
        var groupItems = group.querySelectorAll(".tracker-item");
        groupItems.forEach(function (it) {
          if (it.style.display !== "none") hasVisible = true;
        });
        group.style.display = hasVisible ? "block" : "none";
      });

      if (counterEl) {
        counterEl.textContent = visibleCount;
      }
    }

    var domainCards = document.querySelectorAll(".domain-card");
    domainCards.forEach(function (card) {
      card.addEventListener("click", function () {
        var domain = card.getAttribute("data-domain");
        var isCurrentlyActive = card.classList.contains("active");
        domainCards.forEach(function (c) {
          c.classList.remove("active");
        });
        if (isCurrentlyActive) {
          currentCategory = "all";
        } else {
          card.classList.add("active");
          currentCategory = domain || "all";
        }
        categoryPills.forEach(function (p) {
          p.classList.toggle("active", p.getAttribute("data-filter") === currentCategory);
        });
        applyFilter();
        var toolbar = document.querySelector(".parity-toolbar");
        if (toolbar) {
          toolbar.scrollIntoView({ behavior: "smooth", block: "nearest" });
        }
      });
    });

    categoryPills.forEach(function (pill) {
      pill.addEventListener("click", function () {
        categoryPills.forEach(function (p) {
          p.classList.remove("active");
        });
        pill.classList.add("active");
        currentCategory = pill.getAttribute("data-filter") || "all";
        domainCards.forEach(function (c) {
          c.classList.toggle("active", c.getAttribute("data-domain") === currentCategory);
        });
        applyFilter();
      });
    });

    statusPills.forEach(function (pill) {
      pill.addEventListener("click", function () {
        statusPills.forEach(function (p) {
          p.classList.remove("active");
        });
        pill.classList.add("active");
        currentStatus = pill.getAttribute("data-filter") || "all";
        applyFilter();
      });
    });

    if (searchInput) {
      searchInput.addEventListener("input", function () {
        currentSearch = searchInput.value.trim().toLowerCase();
        applyFilter();
      });

      document.addEventListener("keydown", function (event) {
        var active = document.activeElement;
        var isInput = active && (active.tagName === "INPUT" || active.tagName === "TEXTAREA");
        if (event.key === "/" && !isInput) {
          event.preventDefault();
          searchInput.focus();
        }
      });
    }
  }

  function initPlatformDetection() {
    var cards = document.querySelectorAll(".download-card[data-platform]");
    if (!cards.length) return;

    var ua = navigator.userAgent || "";
    var platform = (navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || "";
    var detected = "linux";

    if (/Win/i.test(ua) || /Win/i.test(platform)) {
      detected = "windows";
    } else if (/Mac/i.test(ua) || /Mac/i.test(platform)) {
      detected = "macos";
    } else if (/Linux|X11/i.test(ua) || /Linux/i.test(platform)) {
      detected = "linux";
    }

    cards.forEach(function (card) {
      var isTarget = card.getAttribute("data-platform") === detected;
      card.classList.toggle("recommended", isTarget);
      var badge = card.querySelector(".detected-badge");
      if (badge) {
        badge.hidden = !isTarget;
      }
    });
  }

  function initOsGuideTabs() {
    var container = document.querySelector(".os-guide-container");
    if (!container) return;

    var tabs = Array.prototype.slice.call(container.querySelectorAll(".os-guide-tab"));
    var panes = container.querySelectorAll(".os-guide-pane");

    function selectTab(tab) {
      var targetId = tab.getAttribute("data-target");
      tabs.forEach(function (t) {
        var active = t === tab;
        t.classList.toggle("active", active);
        t.setAttribute("aria-selected", active ? "true" : "false");
      });
      panes.forEach(function (p) {
        var isActive = p.id === targetId;
        p.classList.toggle("active", isActive);
        if (p.hasAttribute("hidden") || isActive) {
          p.hidden = !isActive;
        }
      });
    }

    tabs.forEach(function (tab) {
      tab.addEventListener("click", function () {
        selectTab(tab);
      });
    });
  }

  function initGameplayPlayer() {
    var video = document.getElementById("gameplayVideo");
    var playBtn = document.getElementById("videoPlayPauseBtn");
    var fsBtn = document.getElementById("videoFullscreenBtn");
    var wrap = document.getElementById("gameplayViewport");
    if (!video || !playBtn || !fsBtn) return;

    function updatePlayButton() {
      if (video.paused) {
        playBtn.textContent = "Play";
      } else {
        playBtn.textContent = "Pause";
      }
    }

    playBtn.addEventListener("click", function () {
      if (video.paused) {
        video.play();
      } else {
        video.pause();
      }
      updatePlayButton();
    });

    video.addEventListener("play", updatePlayButton);
    video.addEventListener("pause", updatePlayButton);

    fsBtn.addEventListener("click", function () {
      var target = wrap || video;
      if (!document.fullscreenElement) {
        if (target.requestFullscreen) {
          target.requestFullscreen();
        } else if (target.webkitRequestFullscreen) {
          target.webkitRequestFullscreen();
        }
      } else {
        if (document.exitFullscreen) {
          document.exitFullscreen();
        }
      }
    });

    document.addEventListener("fullscreenchange", function () {
      if (document.fullscreenElement) {
        fsBtn.textContent = "Exit Full";
      } else {
        fsBtn.textContent = "Fullscreen";
      }
    });
  }

  function initPommeRunner() {
    var canvas = document.getElementById("pommeRunnerCanvas");
    var container = document.getElementById("pommeRunnerCard");
    if (!canvas || !container) return;

    var ctx = canvas.getContext("2d");
    if (!ctx) return;

    var curScoreEl = document.getElementById("runnerCurScore");
    var hiScoreEl = document.getElementById("runnerHiScore");
    var restartBtn = document.getElementById("runnerRestartBtn");
    var biomeBadgeEl = document.getElementById("runnerBiomeBadge");

    canvas.width = 860;
    canvas.height = 220;

    var basePath = (function () {
      var s = document.querySelector('script[src*="app.js"]');
      if (s) {
        var src = s.getAttribute("src") || "";
        var idx = src.indexOf("/assets/");
        if (idx !== -1) return src.slice(0, idx);
      }
      return "";
    })();

    var appleImg = new Image();
    appleImg.src = basePath + "/assets/brand/pomme.png";

    var goldAppleImg = new Image();
    goldAppleImg.src = basePath + "/assets/brand/pomme_gold.png";

    var audioCtx = null;
    function getAudio() {
      if (!audioCtx && (window.AudioContext || window.webkitAudioContext)) {
        var AudioCtor = window.AudioContext || window.webkitAudioContext;
        audioCtx = new AudioCtor();
      }
      return audioCtx;
    }

    function soundJump() {
      try {
        var actx = getAudio();
        if (!actx) return;
        if (actx.state === "suspended") actx.resume();
        var osc = actx.createOscillator();
        var gain = actx.createGain();
        osc.type = "square";
        osc.frequency.setValueAtTime(140, actx.currentTime);
        osc.frequency.exponentialRampToValueAtTime(380, actx.currentTime + 0.09);
        gain.gain.setValueAtTime(0.08, actx.currentTime);
        gain.gain.exponentialRampToValueAtTime(0.001, actx.currentTime + 0.09);
        osc.connect(gain);
        gain.connect(actx.destination);
        osc.start();
        osc.stop(actx.currentTime + 0.09);
      } catch (e) {}
    }

    function soundMilestone() {
      try {
        var actx = getAudio();
        if (!actx) return;
        if (actx.state === "suspended") actx.resume();
        [784, 988].forEach(function (freq, i) {
          var osc = actx.createOscillator();
          var gain = actx.createGain();
          var t = actx.currentTime + i * 0.08;
          osc.type = "square";
          osc.frequency.setValueAtTime(freq, t);
          gain.gain.setValueAtTime(0.07, t);
          gain.gain.exponentialRampToValueAtTime(0.001, t + 0.14);
          osc.connect(gain);
          gain.connect(actx.destination);
          osc.start(t);
          osc.stop(t + 0.14);
        });
      } catch (e) {}
    }

    function soundCoin() {
      try {
        var actx = getAudio();
        if (!actx) return;
        if (actx.state === "suspended") actx.resume();
        [988, 1318].forEach(function (freq, i) {
          var osc = actx.createOscillator();
          var gain = actx.createGain();
          var t = actx.currentTime + i * 0.07;
          osc.type = "sine";
          osc.frequency.setValueAtTime(freq, t);
          gain.gain.setValueAtTime(0.09, t);
          gain.gain.exponentialRampToValueAtTime(0.001, t + 0.16);
          osc.connect(gain);
          gain.connect(actx.destination);
          osc.start(t);
          osc.stop(t + 0.16);
        });
      } catch (e) {}
    }

    function soundTotem() {
      try {
        var actx = getAudio();
        if (!actx) return;
        if (actx.state === "suspended") actx.resume();
        [523.25, 659.25, 783.99, 1046.5].forEach(function (freq, i) {
          var osc = actx.createOscillator();
          var gain = actx.createGain();
          var t = actx.currentTime + i * 0.07;
          osc.type = "sine";
          osc.frequency.setValueAtTime(freq, t);
          gain.gain.setValueAtTime(0.12, t);
          gain.gain.exponentialRampToValueAtTime(0.001, t + 0.25);
          osc.connect(gain);
          gain.connect(actx.destination);
          osc.start(t);
          osc.stop(t + 0.25);
        });
      } catch (e) {}
    }

    function soundCrash() {
      try {
        var actx = getAudio();
        if (!actx) return;
        if (actx.state === "suspended") actx.resume();
        var osc = actx.createOscillator();
        var gain = actx.createGain();
        osc.type = "sawtooth";
        osc.frequency.setValueAtTime(110, actx.currentTime);
        osc.frequency.exponentialRampToValueAtTime(30, actx.currentTime + 0.16);
        gain.gain.setValueAtTime(0.18, actx.currentTime);
        gain.gain.exponentialRampToValueAtTime(0.001, actx.currentTime + 0.16);
        osc.connect(gain);
        gain.connect(actx.destination);
        osc.start();
        osc.stop(actx.currentTime + 0.16);
      } catch (e) {}
    }

    var W = 860;
    var H = 220;
    var GROUND_Y = 180;
    var APPLE_SIZE = 40;

    var hiScore = 0;
    try {
      hiScore = parseInt(localStorage.getItem("pomme_runner_hi") || "0", 10) || 0;
    } catch (e) {}

    function padScore(n) {
      var s = Math.floor(n).toString();
      while (s.length < 5) s = "0" + s;
      return s;
    }

    if (hiScoreEl) hiScoreEl.textContent = padScore(hiScore);

    var state = 0; // 0: IDLE, 1: PLAYING, 2: GAME_OVER
    var score = 0;
    var ticks = 0;
    var speed = 3.8;
    var flashTicks = 0;
    var shake = 0;
    var hasTotem = false;
    var totemShieldAnim = 0;
    var currentBiomeIdx = 0;
    var biomeToastTicks = 0;
    var biomeToastText = "";

    var BIOMES = [
      { name: "PLAINS", sky: "#08090d", ground: "#2ecc71", groundSub: "#131722", label: "OVERWORLD // PLAINS" },
      { name: "DESERT", sky: "#0c0a06", ground: "#d97706", groundSub: "#78350f", label: "DESERT BIOME" },
      { name: "NETHER", sky: "#0f0506", ground: "#dc2626", groundSub: "#450a0a", label: "THE NETHER" },
      { name: "THE END", sky: "#07050d", ground: "#fef08a", groundSub: "#2e1065", label: "THE END // VOID" }
    ];

    function getBiome(sc) {
      if (sc < 250) return 0;
      if (sc < 550) return 1;
      if (sc < 900) return 2;
      return 3;
    }

    var apple = {
      x: 80,
      y: GROUND_Y - APPLE_SIZE,
      vy: 0,
      isGrounded: true,
      jump: function () {
        if (state === 0) {
          state = 1;
          this.vy = -10.0;
          this.isGrounded = false;
          soundJump();
          return;
        }
        if (state === 2) {
          resetGame();
          state = 1;
          this.vy = -10.0;
          this.isGrounded = false;
          soundJump();
          return;
        }
        if (this.isGrounded) {
          this.vy = -10.0;
          this.isGrounded = false;
          soundJump();
        }
      },
      fastFall: function () {
        if (!this.isGrounded) {
          this.vy += 3.6;
        }
      },
      update: function () {
        if (!this.isGrounded) {
          this.vy += 0.46;
          this.y += this.vy;
          if (this.y >= GROUND_Y - APPLE_SIZE) {
            var overPit = false;
            for (var i = 0; i < voidPits.length; i++) {
              var p = voidPits[i];
              if (this.x + APPLE_SIZE / 2 >= p.x && this.x + APPLE_SIZE / 2 <= p.x + p.w) {
                overPit = true;
                break;
              }
            }
            if (!overPit) {
              this.y = GROUND_Y - APPLE_SIZE;
              this.vy = 0;
              this.isGrounded = true;
            } else if (this.y > H + 10) {
              if (hasTotem) {
                consumeTotem();
              } else {
                triggerGameOver();
              }
            }
          }
        } else {
          for (var i = 0; i < voidPits.length; i++) {
            var p = voidPits[i];
            if (this.x + APPLE_SIZE - 4 >= p.x && this.x + 4 <= p.x + p.w) {
              this.isGrounded = false;
              this.vy = 2.5;
              break;
            }
          }
        }
      }
    };

    function consumeTotem() {
      hasTotem = false;
      totemShieldAnim = 40;
      shake = 12;
      soundTotem();
      apple.y = GROUND_Y - APPLE_SIZE - 30;
      apple.vy = -9.2;
      apple.isGrounded = false;
      spawnGoldSparkles(apple.x + 20, apple.y + 20);
      popups.push({
        x: apple.x + 20,
        y: apple.y - 12,
        text: "TOTEM ACTIVATED!",
        life: 45,
        maxLife: 45
      });
    }

    var particles = [];
    function spawnDust(x, y, color) {
      if (Math.random() < 0.28) {
        particles.push({
          x: x,
          y: y + APPLE_SIZE - 2,
          vx: -(Math.random() * 1.5 + 1.0),
          vy: -(Math.random() * 0.8),
          size: Math.random() * 2 + 1.5,
          color: color || "rgba(42, 52, 71, 0.6)",
          life: 14,
          maxLife: 14
        });
      }
    }

    function spawnGoldSparkles(x, y) {
      for (var i = 0; i < 14; i++) {
        var angle = Math.random() * Math.PI * 2;
        var spd = Math.random() * 2.8 + 1.2;
        particles.push({
          x: x,
          y: y,
          vx: Math.cos(angle) * spd,
          vy: Math.sin(angle) * spd,
          size: Math.random() * 2.5 + 2,
          color: "rgba(251, 191, 36, 0.95)",
          life: 22,
          maxLife: 22
        });
      }
    }

    function spawnNetherEmbers() {
      if (Math.random() < 0.22) {
        particles.push({
          x: Math.random() * W,
          y: GROUND_Y - 2,
          vx: (Math.random() - 0.5) * 0.8 - speed * 0.3,
          vy: -(Math.random() * 1.6 + 0.8),
          size: Math.random() * 2.5 + 1.5,
          color: (Math.random() < 0.5) ? "#ef4444" : "#f59e0b",
          life: 32,
          maxLife: 32
        });
      }
    }

    function spawnEnderParticles() {
      if (Math.random() < 0.22) {
        particles.push({
          x: Math.random() * W,
          y: Math.random() * GROUND_Y,
          vx: (Math.random() - 0.5) * 0.6 - speed * 0.2,
          vy: (Math.random() - 0.5) * 0.8,
          size: 2.5,
          color: "#c084fc",
          life: 28,
          maxLife: 28
        });
      }
    }

    var popups = [];
    var clouds = [
      { x: 140, y: 32, w: 56, speed: 0.35 },
      { x: 440, y: 52, w: 44, speed: 0.25 },
      { x: 720, y: 26, w: 60, speed: 0.3 }
    ];

    var groundPebbles = [];
    for (var i = 0; i < 28; i++) {
      groundPebbles.push({
        x: Math.random() * W,
        y: GROUND_Y + 4 + Math.random() * 28,
        size: Math.random() * 2.5 + 1
      });
    }

    var obstacles = [];
    var voidPits = [];
    var collectibles = [];
    var nextSpawnDistance = 400;
    var nextCoinDistance = 500;
    var nextPitDistance = 850;
    var nextTotemDistance = 1000;

    function spawnObstacle(bIndex) {
      var isDouble = Math.random() < 0.35 && score > 80;
      var type = (bIndex === 2) ? "nether_fungus" : (bIndex === 3) ? "chorus" : "cactus";

      if (isDouble) {
        obstacles.push({
          x: W + 20,
          y: GROUND_Y - 44,
          w: 46,
          h: 44,
          type: type,
          double: true
        });
      } else {
        obstacles.push({
          x: W + 20,
          y: GROUND_Y - 42,
          w: 22,
          h: 42,
          type: type,
          double: false
        });
      }
      nextSpawnDistance = Math.floor(Math.max(360, 520 - speed * 12) + Math.random() * 260);
    }

    function spawnVoidPit() {
      voidPits.push({
        x: W + 20,
        w: Math.floor(Math.random() * 12 + 46)
      });
      nextPitDistance = Math.floor(900 + Math.random() * 700);
      nextSpawnDistance = Math.max(nextSpawnDistance, 320);
    }

    function spawnGoldenApple() {
      collectibles.push({
        type: "gapple",
        x: W + 20,
        y: GROUND_Y - 78,
        w: 24,
        h: 24
      });
      nextCoinDistance = Math.floor(620 + Math.random() * 420);
    }

    function spawnTotem() {
      if (!hasTotem) {
        collectibles.push({
          type: "totem",
          x: W + 20,
          y: GROUND_Y - 76,
          w: 24,
          h: 28
        });
      }
      nextTotemDistance = Math.floor(1300 + Math.random() * 900);
    }

    function checkCollision(a, o, pad) {
      pad = pad || 5;
      var ax1 = a.x + pad;
      var ay1 = a.y + pad;
      var ax2 = a.x + APPLE_SIZE - pad;
      var ay2 = a.y + APPLE_SIZE - pad;

      var ox1 = o.x + 2;
      var oy1 = o.y + 2;
      var ox2 = o.x + o.w - 2;
      var oy2 = o.y + o.h - 2;

      return !(ax2 < ox1 || ax1 > ox2 || ay2 < oy1 || ay1 > oy2);
    }

    function resetGame() {
      state = 0;
      score = 0;
      ticks = 0;
      speed = 3.8;
      flashTicks = 0;
      shake = 0;
      hasTotem = false;
      totemShieldAnim = 0;
      currentBiomeIdx = 0;
      biomeToastTicks = 0;
      if (biomeBadgeEl) biomeBadgeEl.textContent = BIOMES[0].name;
      apple.y = GROUND_Y - APPLE_SIZE;
      apple.vy = 0;
      apple.isGrounded = true;
      obstacles = [];
      voidPits = [];
      collectibles = [];
      particles = [];
      popups = [];
      nextSpawnDistance = 360;
      nextCoinDistance = 450;
      nextPitDistance = 750;
      nextTotemDistance = 1000;
      if (curScoreEl) curScoreEl.textContent = "00000";
    }

    function triggerGameOver() {
      if (hasTotem) {
        consumeTotem();
        return;
      }
      state = 2;
      shake = 8;
      soundCrash();
      if (score > hiScore) {
        hiScore = Math.floor(score);
        try {
          localStorage.setItem("pomme_runner_hi", hiScore.toString());
        } catch (e) {}
        if (hiScoreEl) hiScoreEl.textContent = padScore(hiScore);
      }
    }

    window.addEventListener("keydown", function (e) {
      var card = document.getElementById("pommeRunnerCard");
      var inView = card && card.getBoundingClientRect().top < window.innerHeight && card.getBoundingClientRect().bottom > 0;

      if (e.code === "Space" || e.code === "ArrowUp" || e.code === "KeyW") {
        if (inView) e.preventDefault();
        apple.jump();
      } else if (e.code === "ArrowDown" || e.code === "KeyS") {
        if (inView) e.preventDefault();
        apple.fastFall();
      }
    });

    canvas.addEventListener("pointerdown", function (e) {
      e.preventDefault();
      apple.jump();
    });

    if (restartBtn) {
      restartBtn.addEventListener("click", function () {
        resetGame();
        state = 1;
        apple.jump();
      });
    }

    function drawPixelCloud(cx, cy, cw, bIdx) {
      ctx.fillStyle = (bIdx === 2) ? "rgba(185, 28, 28, 0.25)" : (bIdx === 3) ? "rgba(147, 51, 234, 0.25)" : "rgba(42, 52, 71, 0.4)";
      ctx.fillRect(cx, cy + 5, cw, 10);
      ctx.fillRect(cx + 8, cy, cw - 16, 15);
    }

    function drawCelestial(bIdx) {
      if (bIdx === 1) {
        ctx.fillStyle = "#fef08a";
        ctx.fillRect(W - 100, 24, 32, 32);
        ctx.fillStyle = "#facc15";
        ctx.fillRect(W - 92, 28, 20, 20);
      } else if (bIdx === 2) {
        ctx.fillStyle = "rgba(239, 68, 68, 0.35)";
        ctx.fillRect(W - 106, 20, 44, 44);
        ctx.fillStyle = "#ef4444";
        ctx.fillRect(W - 98, 28, 28, 28);
      } else if (bIdx === 3) {
        ctx.fillStyle = "#c084fc";
        ctx.fillRect(W - 96, 24, 24, 32);
        ctx.fillStyle = "#f3e8ff";
        ctx.fillRect(W - 92, 28, 16, 24);
      }
    }

    // Golden Apple Sprite (Genuine Golden Pomme Apple 28x28)
    function drawGoldenApple(gx, gy) {
      var bob = Math.sin(ticks * 0.14) * 3.5;
      var y = gy + bob;
      
      // Ambient golden pulse glow
      ctx.fillStyle = "rgba(251, 191, 36, 0.25)";
      ctx.fillRect(gx - 4, y - 4, 36, 36);

      if (goldAppleImg.complete && goldAppleImg.naturalWidth > 0) {
        ctx.imageSmoothingEnabled = false;
        ctx.drawImage(goldAppleImg, gx, y, 28, 28);
      } else {
        ctx.fillStyle = "#eab308";
        ctx.fillRect(gx, y, 28, 28);
      }
    }

    // High-fidelity Minecraft Totem of Undying Sprite (24x28)
    function drawTotem(tx, ty) {
      var bob = Math.sin(ticks * 0.12) * 3;
      var y = ty + bob;

      // Glow halo
      ctx.fillStyle = "rgba(251, 191, 36, 0.2)";
      ctx.fillRect(tx - 3, y - 3, 30, 34);

      // Wings (Gold & Amber)
      ctx.fillStyle = "#d97706";
      ctx.fillRect(tx, y + 10, 24, 7);
      ctx.fillStyle = "#fbbf24";
      ctx.fillRect(tx + 2, y + 8, 20, 8);
      ctx.fillStyle = "#fef08a";
      ctx.fillRect(tx, y + 8, 4, 4);
      ctx.fillRect(tx + 20, y + 8, 4, 4);

      // Head & Torso
      ctx.fillStyle = "#f59e0b";
      ctx.fillRect(tx + 6, y + 3, 12, 22);
      ctx.fillStyle = "#b45309";
      ctx.fillRect(tx + 7, y + 21, 10, 4);

      // Nose / Brow
      ctx.fillStyle = "#78350f";
      ctx.fillRect(tx + 8, y + 6, 8, 2);
      ctx.fillRect(tx + 11, y + 8, 2, 4);

      // Glowing Emerald Eyes
      ctx.fillStyle = "#10b981";
      ctx.fillRect(tx + 7, y + 8, 3, 3);
      ctx.fillRect(tx + 14, y + 8, 3, 3);
      ctx.fillStyle = "#047857";
      ctx.fillRect(tx + 8, y + 9, 1, 1);
      ctx.fillRect(tx + 15, y + 9, 1, 1);
    }

    // High-fidelity Minecraft Cactus with authentic 3D ribs & needles
    function drawCactus(ox, oy, ow, oh, isDouble, type) {
      if (type === "nether_fungus") {
        drawNetherFungus(ox, oy, ow, oh, isDouble);
        return;
      }
      if (type === "chorus") {
        drawChorusPlant(ox, oy, ow, oh, isDouble);
        return;
      }

      function drawStem(x, y, w, h) {
        // Dark outline
        ctx.fillStyle = "#052e16";
        ctx.fillRect(x, y, w, h);

        // Vertical Ribs (Minecraft 4-tone shading)
        var innerW = w - 2;
        var rx = x + 1;
        var ry = y + 1;
        var rh = h - 2;
        var colW = Math.floor(innerW / 4);

        ctx.fillStyle = "#4ade80"; // Light highlight rib
        ctx.fillRect(rx, ry, colW, rh);

        ctx.fillStyle = "#22c55e"; // Main green rib
        ctx.fillRect(rx + colW, ry, colW, rh);

        ctx.fillStyle = "#16a34a"; // Shaded rib
        ctx.fillRect(rx + colW * 2, ry, colW, rh);

        ctx.fillStyle = "#15803d"; // Dark shadow rib
        ctx.fillRect(rx + colW * 3, ry, innerW - colW * 3, rh);

        // Needles / Spikes (black pixel dots)
        ctx.fillStyle = "#022c22";
        for (var nY = y + 6; nY < y + h - 6; nY += 8) {
          ctx.fillRect(x - 1, nY, 2, 2);
          ctx.fillRect(x + w - 1, nY + 4, 2, 2);
          ctx.fillRect(rx + colW + 1, nY + 2, 2, 2);
        }
      }

      function drawBranch(bx, by, bw, bh, leftSide) {
        // Horizontal arm
        ctx.fillStyle = "#052e16";
        ctx.fillRect(bx, by + bh - 6, bw, 6);
        ctx.fillStyle = "#22c55e";
        ctx.fillRect(bx + 1, by + bh - 5, bw - 2, 4);

        // Vertical branch tip
        var tipX = leftSide ? bx : bx + bw - 6;
        ctx.fillStyle = "#052e16";
        ctx.fillRect(tipX, by, 6, bh);
        ctx.fillStyle = leftSide ? "#4ade80" : "#16a34a";
        ctx.fillRect(tipX + 1, by + 1, 4, bh - 2);
      }

      if (isDouble) {
        drawStem(ox + 4, oy, 16, oh);
        drawBranch(ox, oy + 12, 10, 16, true);
        drawStem(ox + 24, oy + 4, 16, oh - 4);
        drawBranch(ox + 34, oy + 10, 12, 18, false);
      } else {
        drawStem(ox + 3, oy, 16, oh);
        drawBranch(ox - 2, oy + 12, 8, 14, true);
        drawBranch(ox + 13, oy + 16, 9, 14, false);
      }
    }

    // High-fidelity Nether Crimson Fungus
    function drawNetherFungus(ox, oy, ow, oh, isDouble) {
      function drawSingle(x, y, w, h) {
        // Creamy Stem
        ctx.fillStyle = "#450a0a";
        ctx.fillRect(x + w / 2 - 4, y + 14, 8, h - 14);
        ctx.fillStyle = "#fecdd3";
        ctx.fillRect(x + w / 2 - 3, y + 15, 6, h - 16);
        ctx.fillStyle = "#fda4af";
        ctx.fillRect(x + w / 2, y + 15, 3, h - 16);

        // Crimson Cap
        ctx.fillStyle = "#7f1d1d";
        ctx.fillRect(x, y + 4, w, 14);
        ctx.fillRect(x + 2, y, w - 4, 18);

        ctx.fillStyle = "#dc2626";
        ctx.fillRect(x + 2, y + 2, w - 4, 12);
        ctx.fillStyle = "#ef4444";
        ctx.fillRect(x + 4, y + 2, w - 8, 6);

        // Warped Turquoise Spores
        ctx.fillStyle = "#06b6d4";
        ctx.fillRect(x + 4, y + 6, 2, 2);
        ctx.fillRect(x + w - 7, y + 8, 3, 2);
        ctx.fillRect(x + w / 2 - 1, y + 4, 2, 2);
      }

      if (isDouble) {
        drawSingle(ox, oy, 22, oh);
        drawSingle(ox + 22, oy + 6, 20, oh - 6);
      } else {
        drawSingle(ox, oy, ow, oh);
      }
    }

    // High-fidelity The End Chorus Plant
    function drawChorusPlant(ox, oy, ow, oh, isDouble) {
      function drawChorus(x, y, w, h) {
        // Purple segmented stalk
        ctx.fillStyle = "#581c87";
        ctx.fillRect(x + w / 2 - 4, y + 8, 8, h - 8);
        ctx.fillStyle = "#a855f7";
        ctx.fillRect(x + w / 2 - 3, y + 9, 6, h - 10);
        ctx.fillStyle = "#c084fc";
        ctx.fillRect(x + w / 2 - 3, y + 9, 3, h - 10);

        // Segment joints
        ctx.fillStyle = "#3b0764";
        ctx.fillRect(x + w / 2 - 5, y + 20, 10, 2);

        // Chorus Flower on top
        ctx.fillStyle = "#7e22ce";
        ctx.fillRect(x + w / 2 - 7, y, 14, 10);
        ctx.fillStyle = "#fef08a";
        ctx.fillRect(x + w / 2 - 5, y + 2, 10, 6);
        ctx.fillStyle = "#ffffff";
        ctx.fillRect(x + w / 2 - 2, y + 3, 4, 4);
      }

      if (isDouble) {
        drawChorus(ox, oy, 22, oh);
        drawChorus(ox + 22, oy + 4, 20, oh - 4);
      } else {
        drawChorus(ox, oy, ow, oh);
      }
    }

    function loop() {
      ticks++;

      var bIdx = getBiome(score);
      if (bIdx !== currentBiomeIdx) {
        currentBiomeIdx = bIdx;
        biomeToastTicks = 60;
        biomeToastText = BIOMES[bIdx].label;
        if (biomeBadgeEl) biomeBadgeEl.textContent = BIOMES[bIdx].name;
        soundMilestone();
      }

      var biome = BIOMES[bIdx];

      var shakeX = 0;
      var shakeY = 0;
      if (shake > 0) {
        shakeX = (Math.random() - 0.5) * shake;
        shakeY = (Math.random() - 0.5) * shake;
        shake--;
      }

      ctx.save();
      ctx.translate(shakeX, shakeY);

      // Sky Background
      ctx.fillStyle = biome.sky;
      ctx.fillRect(0, 0, W, H);

      // Ambient particles
      if (bIdx === 2 && state === 1) spawnNetherEmbers();
      if (bIdx === 3 && state === 1) spawnEnderParticles();

      drawCelestial(bIdx);

      // Clouds
      clouds.forEach(function (c) {
        if (state === 1) c.x -= c.speed;
        if (c.x + c.w < 0) c.x = W + 30;
        drawPixelCloud(c.x, c.y, c.w, bIdx);
      });

      // Ground & Void Pits
      ctx.strokeStyle = biome.ground;
      ctx.lineWidth = 2;

      var groundSegments = [{ start: 0, end: W }];
      voidPits.forEach(function (pit) {
        var newSegs = [];
        groundSegments.forEach(function (seg) {
          if (pit.x + pit.w < seg.start || pit.x > seg.end) {
            newSegs.push(seg);
          } else {
            if (pit.x > seg.start) newSegs.push({ start: seg.start, end: pit.x });
            if (pit.x + pit.w < seg.end) newSegs.push({ start: pit.x + pit.w, end: seg.end });
          }
        });
        groundSegments = newSegs;
      });

      ctx.beginPath();
      groundSegments.forEach(function (seg) {
        ctx.moveTo(seg.start, GROUND_Y);
        ctx.lineTo(seg.end, GROUND_Y);
      });
      ctx.stroke();

      // Draw Void Pits with Bedrock Layers & Glow
      voidPits.forEach(function (pit) {
        ctx.fillStyle = "#020305";
        ctx.fillRect(pit.x, GROUND_Y, pit.w, H - GROUND_Y);

        // Bedrock cross-section edges
        ctx.fillStyle = "#1e293b";
        ctx.fillRect(pit.x - 3, GROUND_Y, 3, 22);
        ctx.fillRect(pit.x + pit.w, GROUND_Y, 3, 22);
        ctx.fillStyle = "#0f172a";
        ctx.fillRect(pit.x - 2, GROUND_Y + 4, 2, 12);
        ctx.fillRect(pit.x + pit.w, GROUND_Y + 8, 2, 10);

        // Bottom Glow (Lava in Nether, Void in End)
        var glowColor = (bIdx === 2) ? "rgba(239, 68, 68, 0.45)" : "rgba(168, 85, 247, 0.45)";
        ctx.fillStyle = glowColor;
        ctx.fillRect(pit.x + 2, H - 12, pit.w - 4, 12);
      });

      // Ground pebbles
      ctx.fillStyle = biome.groundSub;
      groundPebbles.forEach(function (p) {
        if (state === 1) {
          p.x -= speed;
          if (p.x < 0) p.x = W + Math.random() * 20;
        }
        ctx.fillRect(p.x, p.y, p.size, p.size);
      });

      // Obstacles update & collision
      if (state === 1) {
        nextSpawnDistance -= speed;
        if (nextSpawnDistance <= 0) {
          spawnObstacle(bIdx);
        }

        if (score > 120) {
          nextPitDistance -= speed;
          if (nextPitDistance <= 0) {
            spawnVoidPit();
          }
        }

        nextCoinDistance -= speed;
        if (nextCoinDistance <= 0) {
          spawnGoldenApple();
        }

        if (score > 180) {
          nextTotemDistance -= speed;
          if (nextTotemDistance <= 0) {
            spawnTotem();
          }
        }

        // Void pits update
        for (var pI = voidPits.length - 1; pI >= 0; pI--) {
          voidPits[pI].x -= speed;
          if (voidPits[pI].x + voidPits[pI].w < -40) {
            voidPits.splice(pI, 1);
          }
        }

        // Obstacles update
        for (var i = obstacles.length - 1; i >= 0; i--) {
          var obs = obstacles[i];
          obs.x -= speed;

          if (checkCollision(apple, obs, 5)) {
            triggerGameOver();
          }

          if (obs.x + obs.w < -20) {
            obstacles.splice(i, 1);
          }
        }

        // Collectibles update
        for (var cIdx = collectibles.length - 1; cIdx >= 0; cIdx--) {
          var item = collectibles[cIdx];
          item.x -= speed;

          if (checkCollision(apple, item, 3)) {
            if (item.type === "gapple") {
              score += 50;
              soundCoin();
              spawnGoldSparkles(item.x + 12, item.y + 12);
              popups.push({ x: item.x + 12, y: item.y - 12, text: "+50", life: 35, maxLife: 35 });
            } else if (item.type === "totem") {
              hasTotem = true;
              soundTotem();
              spawnGoldSparkles(item.x + 12, item.y + 12);
              popups.push({ x: item.x + 12, y: item.y - 14, text: "+TOTEM SHIELD", life: 45, maxLife: 45 });
            }
            collectibles.splice(cIdx, 1);
            continue;
          }

          if (item.x + item.w < -20) {
            collectibles.splice(cIdx, 1);
          }
        }
      }

      // Draw obstacles
      obstacles.forEach(function (obs) {
        drawCactus(obs.x, obs.y, obs.w, obs.h, obs.double, obs.type);
      });

      // Draw collectibles
      collectibles.forEach(function (c) {
        if (c.type === "gapple") drawGoldenApple(c.x, c.y);
        else if (c.type === "totem") drawTotem(c.x, c.y);
      });

      // Update apple physics
      if (state === 1) {
        apple.update();
        if (apple.isGrounded) {
          spawnDust(apple.x, apple.y, (bIdx === 2) ? "#ef4444" : (bIdx === 1) ? "#d97706" : "#2ecc71");
        }
      }

      // Draw Particles
      for (var pIdx = particles.length - 1; pIdx >= 0; pIdx--) {
        var pt = particles[pIdx];
        pt.x += pt.vx;
        pt.y += pt.vy;
        pt.life--;
        if (pt.life <= 0) {
          particles.splice(pIdx, 1);
          continue;
        }
        ctx.fillStyle = pt.color;
        ctx.fillRect(pt.x, pt.y, pt.size, pt.size);
      }

      // Draw Apple Drop Shadow on Ground
      if (apple.y < GROUND_Y - 5) {
        var shadowScale = Math.max(0.3, 1 - (GROUND_Y - APPLE_SIZE - apple.y) / 120);
        ctx.fillStyle = "rgba(0, 0, 0, " + (0.35 * shadowScale) + ")";
        ctx.beginPath();
        ctx.ellipse(apple.x + APPLE_SIZE / 2, GROUND_Y + 1, (APPLE_SIZE / 2) * shadowScale, 3 * shadowScale, 0, 0, Math.PI * 2);
        ctx.fill();
      }

      // Floating Score Popups
      for (var popIdx = popups.length - 1; popIdx >= 0; popIdx--) {
        var pop = popups[popIdx];
        pop.y -= 0.8;
        pop.life--;
        if (pop.life <= 0) {
          popups.splice(popIdx, 1);
          continue;
        }
        var popAlpha = pop.life / pop.maxLife;
        ctx.font = "bold 14px monospace";
        ctx.textAlign = "center";
        ctx.fillStyle = "rgba(251, 191, 36, " + popAlpha + ")";
        ctx.fillText(pop.text, pop.x, pop.y);
      }

      // Draw Apple
      ctx.save();
      var appleRenderY = apple.y;
      if (state === 0) {
        appleRenderY += Math.sin(ticks * 0.1) * 2;
      }
      ctx.translate(apple.x + APPLE_SIZE / 2, appleRenderY + APPLE_SIZE / 2);

      if (state === 1) {
        if (!apple.isGrounded) {
          var tilt = Math.max(Math.min(apple.vy * 0.04, 0.35), -0.35);
          ctx.rotate(tilt);
        } else {
          ctx.rotate(Math.sin(ticks * 0.22) * 0.04);
        }
      } else if (state === 2) {
        ctx.rotate(Math.PI / 2);
      }

      // Totem golden glow shield aura
      if (hasTotem) {
        ctx.fillStyle = "rgba(251, 191, 36, 0.28)";
        ctx.beginPath();
        ctx.arc(0, 0, APPLE_SIZE / 2 + 6, 0, Math.PI * 2);
        ctx.fill();
      }

      if (appleImg.complete && appleImg.naturalWidth > 0) {
        ctx.imageSmoothingEnabled = false;
        ctx.drawImage(appleImg, -APPLE_SIZE / 2, -APPLE_SIZE / 2, APPLE_SIZE, APPLE_SIZE);
      } else {
        var as = APPLE_SIZE;
        var ahs = as / 2;
        ctx.fillStyle = "#ef4444";
        ctx.fillRect(-ahs + 4, -ahs + 6, as - 8, as - 8);
        ctx.fillRect(-ahs + 2, -ahs + 10, as - 4, as - 16);
        ctx.fillStyle = "#2ecc71";
        ctx.fillRect(0, -ahs, 8, 6);
        ctx.fillStyle = "#78350f";
        ctx.fillRect(-3, -ahs, 4, 7);
        ctx.fillStyle = "#fee2e2";
        ctx.fillRect(-ahs + 6, -ahs + 8, 4, 4);
      }
      ctx.restore();

      // Totem Shield Flash Animation
      if (totemShieldAnim > 0) {
        totemShieldAnim--;
        var fAlpha = (totemShieldAnim / 40) * 0.45;
        ctx.fillStyle = "rgba(251, 191, 36, " + fAlpha + ")";
        ctx.fillRect(0, 0, W, H);
      }

      // HUD: Biome Notification Banner
      if (biomeToastTicks > 0) {
        biomeToastTicks--;
        var bAlpha = Math.min(biomeToastTicks / 15, 1.0);
        ctx.font = "bold 13px monospace";
        ctx.textAlign = "center";
        ctx.fillStyle = "rgba(46, 204, 113, " + bAlpha + ")";
        ctx.fillText(biomeToastText, W / 2, 42);
      }

      // HUD: Totem Active Indicator in top-left
      if (hasTotem) {
        ctx.font = "bold 12px monospace";
        ctx.textAlign = "left";
        ctx.fillStyle = "#fbbf24";
        ctx.fillText("✦ TOTEM ACTIVE", 18, 28);
      }

      // UI text overlays
      if (state === 0) {
        ctx.font = "bold 14px monospace";
        ctx.textAlign = "center";
        ctx.fillStyle = "#2ecc71";
        ctx.fillText("PRESS [SPACE] OR TAP TO RUN", W / 2, 110);
      } else if (state === 2) {
        ctx.font = "bold 22px monospace";
        ctx.textAlign = "center";
        ctx.fillStyle = "#ef4444";
        ctx.fillText("G A M E   O V E R", W / 2, 95);
        ctx.font = "bold 13px monospace";
        ctx.fillStyle = (ticks % 30 < 15) ? "#f0f3fa" : "#8e9bb0";
        ctx.fillText("PRESS [SPACE] OR TAP TO RETRY", W / 2, 125);
      }

      // Score
      if (state === 1) {
        score += 0.14;
        speed = 3.8 + Math.min(score / 750, 3.6);

        var curInt = Math.floor(score);
        if (curInt > 0 && curInt % 100 === 0 && flashTicks === 0) {
          flashTicks = 28;
          soundMilestone();
        }

        if (curScoreEl) {
          if (flashTicks > 0) {
            flashTicks--;
            curScoreEl.style.opacity = (flashTicks % 8 < 4) ? "0.2" : "1";
            curScoreEl.style.color = "#38bdf8";
          } else {
            curScoreEl.style.opacity = "1";
            curScoreEl.style.color = "var(--accent-green)";
          }
          curScoreEl.textContent = padScore(score);
        }
      }

      ctx.restore();
      requestAnimationFrame(loop);
    }

    requestAnimationFrame(loop);
  }

  function initAll() {
    try { initCounters(); } catch (e) {}
    try { initClipboard(); } catch (e) {}
    try { initSubsystemTabs(); } catch (e) {}
    try { initBootRace(); } catch (e) {}
    try { initTrackerFilter(); } catch (e) {}
    try { initPlatformDetection(); } catch (e) {}
    try { initOsGuideTabs(); } catch (e) {}
    try { initPommeRunner(); } catch (e) {}
    try { initGameplayPlayer(); } catch (e) {}
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initAll);
  } else {
    initAll();
  }
})();
