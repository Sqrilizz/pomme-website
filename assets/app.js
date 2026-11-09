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

  function initAll() {
    initCounters();
    initClipboard();
    initSubsystemTabs();
    initBootRace();
    initTrackerFilter();
    initPlatformDetection();
    initOsGuideTabs();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initAll);
  } else {
    initAll();
  }
})();
