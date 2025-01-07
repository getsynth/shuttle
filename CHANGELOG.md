# Changelog

<!-- Install git-cliff and use `cargo make changelog 0.X.Y` to update this file -->

This is an auto-generated changelog based on commits on the main branch, made with [git-cliff](https://github.com/orhun/git-cliff).
See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

**For proper release notes with more details such as upgrading guidelines, check out the [releases page](https://github.com/shuttle-hq/shuttle/releases).**

## [0.50.0](https://github.com/shuttle-hq/shuttle/compare/v0.49.0..0.50.0) - 2025-01-07

### Features

- Deprecate new deployments ([#1934](https://github.com/shuttle-hq/shuttle/issues/1934)) - ([6ece318](https://github.com/shuttle-hq/shuttle/commit/6ece318b5b1a7bdbb232126346d0ca0794634141))
- Update rds resource type for new platform compatibility ([#1945](https://github.com/shuttle-hq/shuttle/issues/1945)) - ([979a3e0](https://github.com/shuttle-hq/shuttle/commit/979a3e09a56973ac3e0a47e43c6eeda74712e24d))
- Add xl and xxl compute tiers ([#1946](https://github.com/shuttle-hq/shuttle/issues/1946)) - ([84c66ad](https://github.com/shuttle-hq/shuttle/commit/84c66ad0fe179695963fd1b522a9b959f489be62))
- Bump msrv to 1.81, our images to 1.83 ([#1942](https://github.com/shuttle-hq/shuttle/issues/1942)) - ([9925f4e](https://github.com/shuttle-hq/shuttle/commit/9925f4eb27cc66d6b5675e6d4e3b318becfdbd4f))
- Deprecate new project creation ([#1931](https://github.com/shuttle-hq/shuttle/issues/1931)) - ([52525c1](https://github.com/shuttle-hq/shuttle/commit/52525c106601c2363ee35f1626dc6d912c87ec6d))

### Bug Fixes

- *(cargo-shuttle)* Don't override user provided RUST_LOG on local run ([#1930](https://github.com/shuttle-hq/shuttle/issues/1930)) - ([48b8347](https://github.com/shuttle-hq/shuttle/commit/48b83475933d8569d89be9b42f1edfb8bdc3e6bb))
- Admin and cli fixes ([#1922](https://github.com/shuttle-hq/shuttle/issues/1922)) - ([5431b1f](https://github.com/shuttle-hq/shuttle/commit/5431b1fe8a0269518a9f92047d844e62c631a883))
- Cargo-audit version ([#1919](https://github.com/shuttle-hq/shuttle/issues/1919)) - ([2566755](https://github.com/shuttle-hq/shuttle/commit/25667552a78d7323ed6b142b331a24a69fcce60e))

### Documentation

- *(shuttle-service)* Typo fix ([#1935](https://github.com/shuttle-hq/shuttle/issues/1935)) - ([97c35d8](https://github.com/shuttle-hq/shuttle/commit/97c35d8971fa7ee9bbfa70d7e572c0f52dfcf5c1))

### Miscellaneous Tasks

- Update examples - ([4abff0f](https://github.com/shuttle-hq/shuttle/commit/4abff0f3e201dd236db413160c149e6c05f7ca21))
- V0.50.0 - ([f166a87](https://github.com/shuttle-hq/shuttle/commit/f166a87c39e7b38e4abe4682232b613cd956b0d0))

### Miscellaneous

- Update the Turso resource to use libsql 0.6.0 ([#1940](https://github.com/shuttle-hq/shuttle/issues/1940)) - ([cd39fd0](https://github.com/shuttle-hq/shuttle/commit/cd39fd0a0d713bb27573dbc27fbc44f2337b0758))
- Shuttle.dev cleanup ([#1920](https://github.com/shuttle-hq/shuttle/issues/1920)) - ([89affe0](https://github.com/shuttle-hq/shuttle/commit/89affe02b6335584d3b71d498e51b5baebdbf2b9))
- Added proc-macro-error2 dependency ([#1921](https://github.com/shuttle-hq/shuttle/issues/1921)) - ([d81c72b](https://github.com/shuttle-hq/shuttle/commit/d81c72b08afb4fe861a4a752f9a23243a11d8192))

## [0.49.0](https://github.com/shuttle-hq/shuttle/compare/v0.48.3..v0.49.0) - 2024-11-12

### Features

- *(cargo-shuttle)* Redeploy command ([#1916](https://github.com/shuttle-hq/shuttle/issues/1916)) - ([62bf6e8](https://github.com/shuttle-hq/shuttle/commit/62bf6e801210a144afad1a51553d05b9e01e07f0))
- *(cargo-shuttle)* Automatic login via console, login --prompt ([#1913](https://github.com/shuttle-hq/shuttle/issues/1913)) - ([792e681](https://github.com/shuttle-hq/shuttle/commit/792e6818ab33c5bef80327ec265aaa84cf2f16b9))
- *(cargo-shuttle)* Update project name command ([#1912](https://github.com/shuttle-hq/shuttle/issues/1912)) - ([40ff642](https://github.com/shuttle-hq/shuttle/commit/40ff642e46b5968e66f91894e3199453bd1d86a8))
- *(common)* Project update models ([#1910](https://github.com/shuttle-hq/shuttle/issues/1910)) - ([2ba0046](https://github.com/shuttle-hq/shuttle/commit/2ba004613c3080acc9de8fd0d2f0e5e43aae5aa9))
- Dump db endpoint, fixes, Rust 1.82 ([#1907](https://github.com/shuttle-hq/shuttle/issues/1907)) - ([b3ac098](https://github.com/shuttle-hq/shuttle/commit/b3ac098a47c53beba805695cd1f73e73379b3e4b))
- Compute tier upgrade admin command ([#1906](https://github.com/shuttle-hq/shuttle/issues/1906)) - ([e821c8f](https://github.com/shuttle-hq/shuttle/commit/e821c8f036976de34aa07783798af3d48c8577a0))

### Bug Fixes

- *(api-client)* Update archive endpoint ([#1914](https://github.com/shuttle-hq/shuttle/issues/1914)) - ([c8e6d15](https://github.com/shuttle-hq/shuttle/commit/c8e6d15243dff7c0ae6afbb0f931ba6f9f7928f9))
- *(cargo-shuttle)* Shell completion, refactors ([#1909](https://github.com/shuttle-hq/shuttle/issues/1909)) - ([b91de7a](https://github.com/shuttle-hq/shuttle/commit/b91de7afa79174041e23437dc62f3eb1cdf4a113))
- Don't attempt project linking on local run, create project and link on beta init ([#1911](https://github.com/shuttle-hq/shuttle/issues/1911)) - ([bfc05ec](https://github.com/shuttle-hq/shuttle/commit/bfc05ec8cd687334441a8f7767c497cb3fe24239))

### Miscellaneous Tasks

- V0.49.0 ([#1917](https://github.com/shuttle-hq/shuttle/issues/1917)) - ([cef6c28](https://github.com/shuttle-hq/shuttle/commit/cef6c2802c2ff98e2733306aed5e92372f46c9e5))

## [0.48.3](https://github.com/shuttle-hq/shuttle/compare/v0.48.2..v0.48.3) - 2024-10-10

### Bug Fixes

- *(cargo-shuttle)* Don't print platform info in unrelated commands ([#1902](https://github.com/shuttle-hq/shuttle/issues/1902)) - ([e58d589](https://github.com/shuttle-hq/shuttle/commit/e58d5890af586d8d875894e01f847a8d5f1e1734))
- Shuttle-common 0.48.2 ([#1901](https://github.com/shuttle-hq/shuttle/issues/1901)) - ([0acc7e6](https://github.com/shuttle-hq/shuttle/commit/0acc7e6f3397f78e707f8caf59c0c2709f760aef))

### Miscellaneous Tasks

- Cargo update ([#1904](https://github.com/shuttle-hq/shuttle/issues/1904)) - ([80c8763](https://github.com/shuttle-hq/shuttle/commit/80c8763dbce2760d0cb5eb6116f82b662e6ca98d))

### Miscellaneous

- Allow deprecated chrono method in deprecated backends ([#1905](https://github.com/shuttle-hq/shuttle/issues/1905)) - ([0b3f714](https://github.com/shuttle-hq/shuttle/commit/0b3f714e65dd238ecfea9c4e6815505777106159))

## [0.48.2](https://github.com/shuttle-hq/shuttle/compare/v0.48.1..v0.48.2) - 2024-10-10

### Features

- *(cargo-shuttle)* V0.48.2, colored platform outputs, fixes ([#1886](https://github.com/shuttle-hq/shuttle/issues/1886)) - ([be7b731](https://github.com/shuttle-hq/shuttle/commit/be7b73106bbe90cce78ee8e3504fd36983089a2f))

## [0.48.1](https://github.com/shuttle-hq/shuttle/compare/v0.48.0..v0.48.1) - 2024-10-03

### Features

- *(cargo-shuttle)* Shuttle.toml new key names ([#1895](https://github.com/shuttle-hq/shuttle/issues/1895)) - ([4820498](https://github.com/shuttle-hq/shuttle/commit/48204982cb153c131dfc168bec3d9980ae752af8))
- *(cargo-shuttle)* Beta deploy.deny_dirty config, allow dirty deploys by default ([#1894](https://github.com/shuttle-hq/shuttle/issues/1894)) - ([1e90f20](https://github.com/shuttle-hq/shuttle/commit/1e90f20358e1004b0aea53bdfabfa33c56893543))
- *(installer)* Install shuttle binary ([#1890](https://github.com/shuttle-hq/shuttle/issues/1890)) - ([468ca85](https://github.com/shuttle-hq/shuttle/commit/468ca85668736832db6e355cb8aa772276c77504))

### Bug Fixes

- *(cargo-shuttle)* Beta: create missing project on deploy, fix local run, better project link dialogue ([#1893](https://github.com/shuttle-hq/shuttle/issues/1893)) - ([8efada9](https://github.com/shuttle-hq/shuttle/commit/8efada9a62f95dad41721eabdc88cb80eaa91800))
- Reduce gateway cert renew sleep ([#1888](https://github.com/shuttle-hq/shuttle/issues/1888)) - ([0ccb348](https://github.com/shuttle-hq/shuttle/commit/0ccb3488c4e91dfc98a3a8d3d32eb5df082bed48))
- Otel exporter config ([#1891](https://github.com/shuttle-hq/shuttle/issues/1891)) - ([6386223](https://github.com/shuttle-hq/shuttle/commit/63862238113b5522c9969fc705a4db1d45506776))

### Miscellaneous Tasks

- Bump cargo-shuttle ([#1897](https://github.com/shuttle-hq/shuttle/issues/1897)) - ([d01466e](https://github.com/shuttle-hq/shuttle/commit/d01466e443eb407ecc219e906089004e1e789bbd))

## [0.48.0](https://github.com/shuttle-hq/shuttle/compare/v0.47.0..v0.48.0) - 2024-10-01

### Features

- *(admin)* Renew certs command ([#1880](https://github.com/shuttle-hq/shuttle/issues/1880)) - ([05372f0](https://github.com/shuttle-hq/shuttle/commit/05372f0965d02b3ef4d3718ffe1328c027a78b19))
- *(cargo-shuttle)* Beta project linking ([#1885](https://github.com/shuttle-hq/shuttle/issues/1885)) - ([f3e1a19](https://github.com/shuttle-hq/shuttle/commit/f3e1a1920698d0cb585d2964e209985d798e2a38))
- *(cargo-shuttle)* Add `upgrade` subcommand to run install script ([#1848](https://github.com/shuttle-hq/shuttle/issues/1848)) - ([243daff](https://github.com/shuttle-hq/shuttle/commit/243daffea3101238676423be93c219c52d2dd0bc))
- *(cargo-shuttle)* Shuttle binary ([#1878](https://github.com/shuttle-hq/shuttle/issues/1878)) - ([bb7b6be](https://github.com/shuttle-hq/shuttle/commit/bb7b6bec80a87f50cd853d58382f69e0cac6bc15))
- *(cargo-shuttle)* Beta certificate command ([#1860](https://github.com/shuttle-hq/shuttle/issues/1860)) - ([090725d](https://github.com/shuttle-hq/shuttle/commit/090725dabfef15b26fda9023d13f5a28767044c1))
- *(cargo-shuttle)* Beta account command, cleanup ([#1859](https://github.com/shuttle-hq/shuttle/issues/1859)) - ([5385da7](https://github.com/shuttle-hq/shuttle/commit/5385da70a4d4a52fd0bcf7ab21ba53f20ce457e4))
- *(cargo-shuttle)* Beta updated local run ([#1850](https://github.com/shuttle-hq/shuttle/issues/1850)) - ([25f8a75](https://github.com/shuttle-hq/shuttle/commit/25f8a7547ade46aff0aeb9fb81e16ef642345d7d))
- *(cargo-shuttle)* Beta command fixes, beta deployment list pagination, hide deprecated args, beta raw tables ([#1843](https://github.com/shuttle-hq/shuttle/issues/1843)) - ([9d0661c](https://github.com/shuttle-hq/shuttle/commit/9d0661ce5dcec3553e6b928a09eb7bfbd94cc04d))
- *(common)* Typeshare apierror ([#1874](https://github.com/shuttle-hq/shuttle/issues/1874)) - ([3178e09](https://github.com/shuttle-hq/shuttle/commit/3178e09a0b0eb2262f6b0d0633030b686075c1a6))
- *(common)* Beta models update, add typeshare ([#1871](https://github.com/shuttle-hq/shuttle/issues/1871)) - ([d147890](https://github.com/shuttle-hq/shuttle/commit/d14789082efdca4ac17181abfab1eba5b1908c6d))
- *(common)* Beta certificate models update ([#1865](https://github.com/shuttle-hq/shuttle/issues/1865)) - ([4b8b86d](https://github.com/shuttle-hq/shuttle/commit/4b8b86d1a7d27bcb4b6109f90ba9a07dd30e1d5f))
- *(runtime)* Improve beta tracing ([#1879](https://github.com/shuttle-hq/shuttle/issues/1879)) - ([89b995f](https://github.com/shuttle-hq/shuttle/commit/89b995f40110297cd4f47cb2a9000fe08a072cd6))
- *(runtime)* Beta runtime as client ([#1844](https://github.com/shuttle-hq/shuttle/issues/1844)) - ([f8170be](https://github.com/shuttle-hq/shuttle/commit/f8170be56602b8021bc474f41a7ba4c1bf18ce37))
- Beta models update, fix tables and prints ([#1875](https://github.com/shuttle-hq/shuttle/issues/1875)) - ([6866f62](https://github.com/shuttle-hq/shuttle/commit/6866f62c773f11d5ff64969fb47097c1d3c552d2))
- Disable cargo-shuttle error suggestions for beta ([#1869](https://github.com/shuttle-hq/shuttle/issues/1869)) - ([ab3ec3e](https://github.com/shuttle-hq/shuttle/commit/ab3ec3e70b4f817042a34ab1d4126f5fd32c4f0a))
- Instrument permit calls ([#1858](https://github.com/shuttle-hq/shuttle/issues/1858)) - ([b4abc65](https://github.com/shuttle-hq/shuttle/commit/b4abc658493019411d9ef34b73a62a065bc46f95))
- Rust 1.80 ([#1845](https://github.com/shuttle-hq/shuttle/issues/1845)) - ([e6308e0](https://github.com/shuttle-hq/shuttle/commit/e6308e0b564b78bd49b9ca68a036848150d57055))
- Add `diesel-async` support for `shuttle-aws-rds` ([#1808](https://github.com/shuttle-hq/shuttle/issues/1808)) - ([80892d3](https://github.com/shuttle-hq/shuttle/commit/80892d3e4cc173098ccbd4c84c43b0f695fae5bf))

### Bug Fixes

- *(admin)* Beta set access endpoint ([#1884](https://github.com/shuttle-hq/shuttle/issues/1884)) - ([b9e94c7](https://github.com/shuttle-hq/shuttle/commit/b9e94c7d5cbc2ee63211d0fe06070e83a1d95227))
- *(c-s)* Windows ([#1862](https://github.com/shuttle-hq/shuttle/issues/1862)) - ([553cb9c](https://github.com/shuttle-hq/shuttle/commit/553cb9cb3078bf66c78e6107bd18e83885a85fbe))
- *(deployer)* Revert typo fix ([#1856](https://github.com/shuttle-hq/shuttle/issues/1856)) - ([0ab5350](https://github.com/shuttle-hq/shuttle/commit/0ab535096757ef1356807b0d067740a21d959ad2))
- *(install-script)* Failing install on Arch Linux ([#1810](https://github.com/shuttle-hq/shuttle/issues/1810)) ([#1861](https://github.com/shuttle-hq/shuttle/issues/1861)) - ([85f1000](https://github.com/shuttle-hq/shuttle/commit/85f100056996a7e2564938690fc9a8c32ebe7249))
- *(resource-recorder)* Check project permission by project id ([#1839](https://github.com/shuttle-hq/shuttle/issues/1839)) - ([4b7f3fc](https://github.com/shuttle-hq/shuttle/commit/4b7f3fc05fd355dbd679eb7d5d63d74fe7027a81))
- *(shuttle-common)* Deployment message typo ([#1855](https://github.com/shuttle-hq/shuttle/issues/1855)) - ([b8aa727](https://github.com/shuttle-hq/shuttle/commit/b8aa727b1ae90ea8d39fe60f887ca472621d1de8))
- Beta local run resource fixes ([#1876](https://github.com/shuttle-hq/shuttle/issues/1876)) - ([793f2b1](https://github.com/shuttle-hq/shuttle/commit/793f2b198f6b8b0ffcf33a2b65ea3ba86101db09))
- Windows backslash paths in zip ([#1877](https://github.com/shuttle-hq/shuttle/issues/1877)) - ([00fa268](https://github.com/shuttle-hq/shuttle/commit/00fa26853247eba4a30bbce5663dd4892a91548f))
- Remove blanket suggestion to open ticket ([#1870](https://github.com/shuttle-hq/shuttle/issues/1870)) - ([f3e0236](https://github.com/shuttle-hq/shuttle/commit/f3e02363c28029c2002e6c46967321d4ea84007a))
- Remove teams call for beta project list ([#1872](https://github.com/shuttle-hq/shuttle/issues/1872)) - ([651d760](https://github.com/shuttle-hq/shuttle/commit/651d760f378cd18a140f1064976eff4c44ee8e1f))
- Audit ([#1841](https://github.com/shuttle-hq/shuttle/issues/1841)) - ([00e01a0](https://github.com/shuttle-hq/shuttle/commit/00e01a04f8fb64de5caac50d1e765005308915ad))

### Miscellaneous Tasks

- V0.48.0 ([#1887](https://github.com/shuttle-hq/shuttle/issues/1887)) - ([af27275](https://github.com/shuttle-hq/shuttle/commit/af27275ccc881cf814acff757538ea7554afd8ae))
- Deployer rust 1.81 ([#1883](https://github.com/shuttle-hq/shuttle/issues/1883)) - ([1d2ea62](https://github.com/shuttle-hq/shuttle/commit/1d2ea621b763fc1f039f8513d44901c0d75246d2))
- Sqlx 0.8.2, MSRV 1.78 ([#1867](https://github.com/shuttle-hq/shuttle/issues/1867)) - ([a74f4ee](https://github.com/shuttle-hq/shuttle/commit/a74f4ee049f156848f1106d81bc8ef8b8858f7dd))
- Disable honeycomb export, remove vars ([#1868](https://github.com/shuttle-hq/shuttle/issues/1868)) - ([873b172](https://github.com/shuttle-hq/shuttle/commit/873b1720eda277f2b092e757a5826097d4afc7ee))
- Updated beta api url ([#1857](https://github.com/shuttle-hq/shuttle/issues/1857)) - ([049411a](https://github.com/shuttle-hq/shuttle/commit/049411a84fdac7b48265612972d1a7006ace1c84))
- Remove qa workflows ([#1837](https://github.com/shuttle-hq/shuttle/issues/1837)) - ([0753973](https://github.com/shuttle-hq/shuttle/commit/07539737958d23c5ea06924323bd894c572c2f77))

### Miscellaneous

- Bump sqlx to 0.8 in libraries ([#1847](https://github.com/shuttle-hq/shuttle/issues/1847)) - ([91c189a](https://github.com/shuttle-hq/shuttle/commit/91c189a1aa5cd9451eb2146ed8f85b222c4a8423))

## [0.47.0](https://github.com/shuttle-hq/shuttle/compare/v0.46.0..v0.47.0) - 2024-07-22

### Features

- *(backends)* Permit delete project idempotency ([#1806](https://github.com/shuttle-hq/shuttle/issues/1806)) - ([fbc011a](https://github.com/shuttle-hq/shuttle/commit/fbc011a3b646770dbe536c27e60a0a66bd942a3d))
- *(cargo-shuttle)* Deploy --output-archive <path> ([#1831](https://github.com/shuttle-hq/shuttle/issues/1831)) - ([b1a5733](https://github.com/shuttle-hq/shuttle/commit/b1a57333c5e8a1fb80d46d2dcce1c55c64c27d89))
- *(cargo-shuttle)* Beta deployment stop command ([#1830](https://github.com/shuttle-hq/shuttle/issues/1830)) - ([1637845](https://github.com/shuttle-hq/shuttle/commit/16378454626df31628d47b4deb0b0b975a40ea5c))
- *(cargo-shuttle)* Default beta url ([#1829](https://github.com/shuttle-hq/shuttle/issues/1829)) - ([880119e](https://github.com/shuttle-hq/shuttle/commit/880119ef4a117f1a762f107c87e1deea2e943515))
- *(cargo-shuttle)* Outgoing body debug ([#1828](https://github.com/shuttle-hq/shuttle/issues/1828)) - ([1d8a281](https://github.com/shuttle-hq/shuttle/commit/1d8a281eb80fcef8b0651feb65ab118869f1dfe0))
- *(cargo-shuttle)* Basic request tracing ([#1827](https://github.com/shuttle-hq/shuttle/issues/1827)) - ([c537566](https://github.com/shuttle-hq/shuttle/commit/c5375666c4bbd8faa06bb02f87ac3ba99f9d128e))
- *(cargo-shuttle)* Beta archive deployment separation ([#1814](https://github.com/shuttle-hq/shuttle/issues/1814)) - ([d60c310](https://github.com/shuttle-hq/shuttle/commit/d60c310b9993ef9f21e868afb6bf84763ebab6c3))
- *(cargo-shuttle)* Beta: support image deployment ([#1796](https://github.com/shuttle-hq/shuttle/issues/1796)) - ([01f5dca](https://github.com/shuttle-hq/shuttle/commit/01f5dca39aa10f60c3daeb97f977dd412ea1bf1c))
- *(common)* Proxy backend ([#1815](https://github.com/shuttle-hq/shuttle/issues/1815)) - ([52c00c0](https://github.com/shuttle-hq/shuttle/commit/52c00c0223bcb7bd59523a4724f2077860aaf82c))
- OpenAI resource ([#1794](https://github.com/shuttle-hq/shuttle/issues/1794)) - ([e2e178c](https://github.com/shuttle-hq/shuttle/commit/e2e178cf0b98c75a005382fc29535abaebe771ba))
- Shuttle-api-client ([#1833](https://github.com/shuttle-hq/shuttle/issues/1833)) - ([02ba42d](https://github.com/shuttle-hq/shuttle/commit/02ba42dba49a393ed4b94d82d96298478fba5bcb))
- Beta runtime version field ([#1826](https://github.com/shuttle-hq/shuttle/issues/1826)) - ([f72134c](https://github.com/shuttle-hq/shuttle/commit/f72134cbce2e0707dcaffcee639529c82aa2e3fc))
- Beta deploy follow status, show logs on failure ([#1825](https://github.com/shuttle-hq/shuttle/issues/1825)) - ([d70e715](https://github.com/shuttle-hq/shuttle/commit/d70e715f833ef5f62e353bcb3ed25cf71bc4e606))
- Beta UX fixes ([#1824](https://github.com/shuttle-hq/shuttle/issues/1824)) - ([5f18bd1](https://github.com/shuttle-hq/shuttle/commit/5f18bd11a25734897c3b9e4cbb7e270201dff58c))
- Beta multi lang deployment models ([#1820](https://github.com/shuttle-hq/shuttle/issues/1820)) - ([ec0df80](https://github.com/shuttle-hq/shuttle/commit/ec0df80483ec5ff2f67e7ee2d5405e6a61ad26f0))
- Beta /me endpoint & project name check ([#1819](https://github.com/shuttle-hq/shuttle/issues/1819)) - ([11850df](https://github.com/shuttle-hq/shuttle/commit/11850df4373a1a3a18155334365de8891fec7abe))
- Implement rds resource on beta platform ([#1812](https://github.com/shuttle-hq/shuttle/issues/1812)) - ([22dc7c7](https://github.com/shuttle-hq/shuttle/commit/22dc7c71a3ab7bf86481b8fce0bb0286dd614bce))
- Beta builder args ([#1813](https://github.com/shuttle-hq/shuttle/issues/1813)) - ([9fe7e56](https://github.com/shuttle-hq/shuttle/commit/9fe7e56a26b36e10a5e26ffe9c20dbce6148509e))
- Filter permit project based on platform ([#1811](https://github.com/shuttle-hq/shuttle/issues/1811)) - ([7c7f0f1](https://github.com/shuttle-hq/shuttle/commit/7c7f0f104b07a77398976cffebe41e143cb6d144))
- Headroom provisioner models ([#1795](https://github.com/shuttle-hq/shuttle/issues/1795)) - ([0de2730](https://github.com/shuttle-hq/shuttle/commit/0de2730c5881ef6f7a6acaf36e4703d36be45064))

### Bug Fixes

- *(auth)* Local development ([#1809](https://github.com/shuttle-hq/shuttle/issues/1809)) - ([f0e770f](https://github.com/shuttle-hq/shuttle/commit/f0e770ff2cdc937cb8d58f6d46df037e147e52dd))
- *(auth)* Migrate in tests ([#1805](https://github.com/shuttle-hq/shuttle/issues/1805)) - ([3b5e9f8](https://github.com/shuttle-hq/shuttle/commit/3b5e9f8e6037d28da1c29abddc0b1f2d218c043c))
- *(backends)* Team project list ([#1816](https://github.com/shuttle-hq/shuttle/issues/1816)) - ([1f88208](https://github.com/shuttle-hq/shuttle/commit/1f88208817e6fcc50c0a81b830d903e11703814e))
- *(gateway)* Drop tower-sanitize-path ([#1832](https://github.com/shuttle-hq/shuttle/issues/1832)) - ([21a8e22](https://github.com/shuttle-hq/shuttle/commit/21a8e228e5191d2b45a021a3a744b5d3547fc622))
- Audit ([#1834](https://github.com/shuttle-hq/shuttle/issues/1834)) - ([234476a](https://github.com/shuttle-hq/shuttle/commit/234476a533e558abdc868eb7d3f92e3aa0c1b472))

### Miscellaneous Tasks

- *(auth)* Don't migrate pool ([#1803](https://github.com/shuttle-hq/shuttle/issues/1803)) - ([03406f0](https://github.com/shuttle-hq/shuttle/commit/03406f0e8aa3368b3b7ff4bc8d798a4dc0fae237))
- *(shuttle-qdrant)* Update to qdrant 1.10 ([#1821](https://github.com/shuttle-hq/shuttle/issues/1821)) - ([69fb9ed](https://github.com/shuttle-hq/shuttle/commit/69fb9ed4d66669cee84a9c0a090cc58d137ea600))
- 0.47.0 ([#1835](https://github.com/shuttle-hq/shuttle/issues/1835)) - ([bcd78d2](https://github.com/shuttle-hq/shuttle/commit/bcd78d294692cfef53e8d24a4e602baf5427355d))
- Bump bytes to resolve audit ([#1822](https://github.com/shuttle-hq/shuttle/issues/1822)) - ([78373d4](https://github.com/shuttle-hq/shuttle/commit/78373d4d4e5c4fb7b66ce5e0492052938075dc2a))
- Bump gix, cargo update ([#1817](https://github.com/shuttle-hq/shuttle/issues/1817)) - ([90c51de](https://github.com/shuttle-hq/shuttle/commit/90c51de31c54d14e5e9ebba299b097dd7141504f))
- Rust 1.79, MSRV 1.77, cleanup ([#1800](https://github.com/shuttle-hq/shuttle/issues/1800)) - ([6fb9a85](https://github.com/shuttle-hq/shuttle/commit/6fb9a856a65c65ae78f7f3b0c7bff1fcf24ff82d))

## [0.46.0](https://github.com/shuttle-hq/shuttle/compare/v0.45.0..v0.46.0) - 2024-06-13

### Features

- *(auth)* Add has_access_to_beta column ([#1779](https://github.com/shuttle-hq/shuttle/issues/1779)) - ([47f282b](https://github.com/shuttle-hq/shuttle/commit/47f282bdbbc0f20af2411efea6ff6654901041c9))
- *(cargo-shuttle)* Beta deploy endpoint ([#1777](https://github.com/shuttle-hq/shuttle/issues/1777)) - ([6ef0312](https://github.com/shuttle-hq/shuttle/commit/6ef0312b152ae8b7372e15e0586eda84464d61c5))
- *(cargo-shuttle)* Beta compact project and deployment tables ([#1778](https://github.com/shuttle-hq/shuttle/issues/1778)) - ([baeddf8](https://github.com/shuttle-hq/shuttle/commit/baeddf8f451410936bfa7f26297e3a7bae173b75))
- Use 'shuttle' feature flag if it exists ([#1799](https://github.com/shuttle-hq/shuttle/issues/1799)) - ([d4c9500](https://github.com/shuttle-hq/shuttle/commit/d4c95000b757b89cd5f732c79939e57f9f28163b))
- Beta deployment response, current deployment & logs ([#1782](https://github.com/shuttle-hq/shuttle/issues/1782)) - ([e5b20d7](https://github.com/shuttle-hq/shuttle/commit/e5b20d7c036751cc7986eb5181b84393af480268))
- Update beta logs logic ([#1775](https://github.com/shuttle-hq/shuttle/issues/1775)) - ([60e2d26](https://github.com/shuttle-hq/shuttle/commit/60e2d267dcee163ca46f67fe1841558557afaa37))
- List team projects on beta ([#1772](https://github.com/shuttle-hq/shuttle/issues/1772)) - ([f87b873](https://github.com/shuttle-hq/shuttle/commit/f87b8736969b32f6085836edde03f9f750800eaa))

### Bug Fixes

- *(backends)* Increase permit per page count ([#1781](https://github.com/shuttle-hq/shuttle/issues/1781)) - ([2d9f7d6](https://github.com/shuttle-hq/shuttle/commit/2d9f7d65d31e533e1b1f770ae68ebb2972858fc6))
- *(cargo-shuttle)* Delete command url ([#1798](https://github.com/shuttle-hq/shuttle/issues/1798)) - ([c64cab0](https://github.com/shuttle-hq/shuttle/commit/c64cab0fe6afd73aef1f1a2185aec1ee77f320b8))
- *(gateway)* Use req.path in bouncer ([#1797](https://github.com/shuttle-hq/shuttle/issues/1797)) - ([2a7e5ce](https://github.com/shuttle-hq/shuttle/commit/2a7e5ce1aacb0f0de6b47403c6eb5fe887492ef2))
- Improve delete command reliability ([#1791](https://github.com/shuttle-hq/shuttle/issues/1791)) - ([7832610](https://github.com/shuttle-hq/shuttle/commit/78326105459e9a988be0682eeb0fcdee1c880ef3))
- Recursive permissionsdal impl ([#1785](https://github.com/shuttle-hq/shuttle/issues/1785)) - ([7d1c9db](https://github.com/shuttle-hq/shuttle/commit/7d1c9db6b59a5b253eaf82f89238b0f7096ad2e5))

### Refactor

- Allow cors from preview environments ([#1773](https://github.com/shuttle-hq/shuttle/issues/1773)) - ([87245c9](https://github.com/shuttle-hq/shuttle/commit/87245c930eb0109d3c0198304f0c6669ab7289cb))
- Implement permission dal when behind an arc ([#1784](https://github.com/shuttle-hq/shuttle/issues/1784)) - ([5d2311b](https://github.com/shuttle-hq/shuttle/commit/5d2311b90fea7f9b72f4fc5beb7f9e94ea0ac1b3))
- Better errors ([#1783](https://github.com/shuttle-hq/shuttle/issues/1783)) - ([798f44c](https://github.com/shuttle-hq/shuttle/commit/798f44cf41e1019eb78e889eecedcb1d9915fa37))
- Record the correct shuttle.user.id for the new platform ([#1780](https://github.com/shuttle-hq/shuttle/issues/1780)) - ([ac6cf7b](https://github.com/shuttle-hq/shuttle/commit/ac6cf7b5b2917731fd5f4ac254c0728c4cba5561))
- ApiError helpers ([#1776](https://github.com/shuttle-hq/shuttle/issues/1776)) - ([97571c5](https://github.com/shuttle-hq/shuttle/commit/97571c5b535bbbc42f5789fcbe80bb236cfadacf))
- Use list struct for project lists ([#1774](https://github.com/shuttle-hq/shuttle/issues/1774)) - ([7451353](https://github.com/shuttle-hq/shuttle/commit/745135378292b773ef3e1e942d3b8c6dc9c96ced))

### Miscellaneous Tasks

- 0.46.0 ([#1801](https://github.com/shuttle-hq/shuttle/issues/1801)) - ([5519ed9](https://github.com/shuttle-hq/shuttle/commit/5519ed97092d37e399a02c69ce1fca3ba9d2c4d3))
- Bump Rust to 1.78.0 ([#1788](https://github.com/shuttle-hq/shuttle/issues/1788)) - ([934f0f5](https://github.com/shuttle-hq/shuttle/commit/934f0f5ed03a02e643ae26bde5656e519dc853c6))
- Don't restart local otel collector ([#1786](https://github.com/shuttle-hq/shuttle/issues/1786)) - ([608ff28](https://github.com/shuttle-hq/shuttle/commit/608ff281d4012af547e3d9696f767b8e48f60350))
- Windows builds ([#1770](https://github.com/shuttle-hq/shuttle/issues/1770)) - ([01c3763](https://github.com/shuttle-hq/shuttle/commit/01c3763430a690501880984846a38a48454c7d35))

### Miscellaneous

- Remove model used for external provisioner reqs ([#1793](https://github.com/shuttle-hq/shuttle/issues/1793)) - ([36fc318](https://github.com/shuttle-hq/shuttle/commit/36fc318fe180d8b1a31b549ecb24161b7f10fbb2))

## [0.45.0](https://github.com/shuttle-hq/shuttle/compare/v0.44.0..v0.45.0) - 2024-05-09

### Features

- *(cargo-shuttle)* Adapt project response for beta ([#1756](https://github.com/shuttle-hq/shuttle/issues/1756)) - ([7fd5723](https://github.com/shuttle-hq/shuttle/commit/7fd57237cc6253e5a3e8273fcbb8ee5c00251708))
- *(cargo-shuttle)* Nits - ([f1f4b4e](https://github.com/shuttle-hq/shuttle/commit/f1f4b4e4fdee33a147c351c331c06ce824ce9d75))
- *(runtime)* Version endpoint for runner version check - ([3757bfe](https://github.com/shuttle-hq/shuttle/commit/3757bfe55214ee19e9096661e20e4d369a7073f8))

### Bug Fixes

- *(backends)* Switch to http otel collector for compatibility with beta - ([85a8d54](https://github.com/shuttle-hq/shuttle/commit/85a8d54da6f6dd9524f81e13ba7333fce516db61))
- *(cargo-shuttle)* Better error when docker is not running ([#1759](https://github.com/shuttle-hq/shuttle/issues/1759)) - ([873f11b](https://github.com/shuttle-hq/shuttle/commit/873f11b0b83c1640185438f0c949d22bda4af1de))
- *(cargo-shuttle)* Skip org project listing on beta - ([b52786b](https://github.com/shuttle-hq/shuttle/commit/b52786b96e8ecee7bf6ea3bfd67a17e832346618))
- *(runtime)* Make compatible with alpha & beta - ([3fc73ea](https://github.com/shuttle-hq/shuttle/commit/3fc73ea9c788f40f099e5b91d73724f13674be90))
- *(service)* Improve workspace member discovery and error ([#1758](https://github.com/shuttle-hq/shuttle/issues/1758)) - ([28f4011](https://github.com/shuttle-hq/shuttle/commit/28f401159808252155118d211f458a2cbeacf259))
- Otel-otlp http client, deployment request ([#1765](https://github.com/shuttle-hq/shuttle/issues/1765)) - ([f947dee](https://github.com/shuttle-hq/shuttle/commit/f947dee6c9041393ff9d36316bf26da955a7ac92))
- Review comments on ecs-common - ([cd2c7a8](https://github.com/shuttle-hq/shuttle/commit/cd2c7a8b7f2f2c73fd8aa77ec0b3e8705970c71a))
- Make services compatible with beta runtime - ([a45addb](https://github.com/shuttle-hq/shuttle/commit/a45addb476cde6aaeab7853963eae3430b156ce2))

### Refactor

- *(cargo-shuttle)* Cleanup of beta branching & behaviour ([#1760](https://github.com/shuttle-hq/shuttle/issues/1760)) - ([43d7f28](https://github.com/shuttle-hq/shuttle/commit/43d7f28440290b8a969232b20a9d6a6584b13a15))
- Return team details when creating a new team ([#1761](https://github.com/shuttle-hq/shuttle/issues/1761)) - ([399b525](https://github.com/shuttle-hq/shuttle/commit/399b525bb658d4d2b122c3008f65d5cf1e686299))
- Rename organizations to teams ([#1754](https://github.com/shuttle-hq/shuttle/issues/1754)) - ([61ec530](https://github.com/shuttle-hq/shuttle/commit/61ec5303f2671a787b7b49e62e568f184a1f2767))

### Testing

- Fix failing logger tests ([#1766](https://github.com/shuttle-hq/shuttle/issues/1766)) - ([935ae26](https://github.com/shuttle-hq/shuttle/commit/935ae266c7721eea43322265e34d1a2b6887fc24))

### Miscellaneous Tasks

- V0.45.0 ([#1767](https://github.com/shuttle-hq/shuttle/issues/1767)) - ([f00a098](https://github.com/shuttle-hq/shuttle/commit/f00a0987206857f2e39a47a5a390f4c8f52b5859))

### Miscellaneous

- Arg for provisioner-uri ([#1763](https://github.com/shuttle-hq/shuttle/issues/1763)) - ([8671518](https://github.com/shuttle-hq/shuttle/commit/8671518d0e7f9183d48bc157f9a75b07f52f5a17))
- Project delete ([#1762](https://github.com/shuttle-hq/shuttle/issues/1762)) - ([cf701a7](https://github.com/shuttle-hq/shuttle/commit/cf701a7d0d597d1a7d127281f758c0272e3944ca))
- Merge pull request #1753 from shuttle-hq/feat/shuttle-ecs-common - ([3322112](https://github.com/shuttle-hq/shuttle/commit/332211226c346f86abe41e3da66770b778cc7a59))
- Update timeout comment - ([95c155b](https://github.com/shuttle-hq/shuttle/commit/95c155bf9504c3f4630f34302c29d93eb16fb488))
- Merge remote-tracking branch 'upstream/main' into feat/shuttle-ecs-common - ([a0e1bf1](https://github.com/shuttle-hq/shuttle/commit/a0e1bf153af7a07a3548680b1f7fea2733c92427))
- Merge remote-tracking branch 'upstream/main' into feat/shuttle-ecs-common - ([ceed492](https://github.com/shuttle-hq/shuttle/commit/ceed492d3543058cecba90a14593757044354ccd))

## [0.44.0](https://github.com/shuttle-hq/shuttle/compare/v0.43.0..v0.44.0) - 2024-04-23

### Features

- *(admin)* Change project owner ([#1725](https://github.com/shuttle-hq/shuttle/issues/1725)) - ([d26e6c8](https://github.com/shuttle-hq/shuttle/commit/d26e6c838ef478b513401e0c9fae77cfb2677840))
- *(c-s beta)* Enable resource delete ([#1745](https://github.com/shuttle-hq/shuttle/issues/1745)) - ([fe9f49c](https://github.com/shuttle-hq/shuttle/commit/fe9f49c2c755f8d4eb32f7c07201f2b99a257271))
- *(cargo-shuttle)* Beta deploy zip file, package name ([#1742](https://github.com/shuttle-hq/shuttle/issues/1742)) - ([81ecaae](https://github.com/shuttle-hq/shuttle/commit/81ecaaea3007c798c9ac1a307a4de4192f79d4c1))
- *(cargo-shuttle)* Beta flag, remove project list pagination logic ([#1732](https://github.com/shuttle-hq/shuttle/issues/1732)) - ([72839f0](https://github.com/shuttle-hq/shuttle/commit/72839f07bf5b92063decae2393016c792852a6a0))
- *(cargo-shuttle beta)* Enable resource list ([#1744](https://github.com/shuttle-hq/shuttle/issues/1744)) - ([82876bd](https://github.com/shuttle-hq/shuttle/commit/82876bdf469c0b0d1430e28a9d01c9a51a1cdd08))
- *(common)* Add building EcsState ([#1752](https://github.com/shuttle-hq/shuttle/issues/1752)) - ([90bd111](https://github.com/shuttle-hq/shuttle/commit/90bd1118d41c2d37c10a96dc52b47eb56071fa4e))
- *(gateway)* Compare permission checks to permit ([#1706](https://github.com/shuttle-hq/shuttle/issues/1706)) - ([9130d5b](https://github.com/shuttle-hq/shuttle/commit/9130d5bab0f3a70c3fcc58c2cf8324102bcbd9a7))
- *(shuttle-turso)* Change turso resource output type to database  ([#1747](https://github.com/shuttle-hq/shuttle/issues/1747)) - ([86ca0c0](https://github.com/shuttle-hq/shuttle/commit/86ca0c0bab05d58a20b8b887571bdc6e4080e289))
- Add `--head` and `--tail` to logs command ([#1629](https://github.com/shuttle-hq/shuttle/issues/1629)) - ([b5116c7](https://github.com/shuttle-hq/shuttle/commit/b5116c73661d0aab1192429fae3aab9d59dcb74f))
- Get project owners and only personal projects ([#1733](https://github.com/shuttle-hq/shuttle/issues/1733)) - ([13b334f](https://github.com/shuttle-hq/shuttle/commit/13b334f2a00b8224be9fee5d27b1c6f1161e443a))
- Support secrets on beta platform ([#1748](https://github.com/shuttle-hq/shuttle/issues/1748)) - ([b84ea8f](https://github.com/shuttle-hq/shuttle/commit/b84ea8fc64c237ce204e5868ab22af1a9d119b6f))
- Add services client constructor for default headers ([#1737](https://github.com/shuttle-hq/shuttle/issues/1737)) - ([ae3c5b0](https://github.com/shuttle-hq/shuttle/commit/ae3c5b03927d06976efacb3f228cb077563a9bfb))
- Add models and header for new ecs provisioner ([#1730](https://github.com/shuttle-hq/shuttle/issues/1730)) - ([d09d803](https://github.com/shuttle-hq/shuttle/commit/d09d8036c7374d009d77f0bfb3ad19f706c1d081))
- Org members ([#1728](https://github.com/shuttle-hq/shuttle/issues/1728)) - ([37d5f6f](https://github.com/shuttle-hq/shuttle/commit/37d5f6f6a7bffd9ab9870fad3cfb6c6d174d0ba1))
- Bump poem version ([#1724](https://github.com/shuttle-hq/shuttle/issues/1724)) - ([639b5ae](https://github.com/shuttle-hq/shuttle/commit/639b5ae78e5704cdd5e5ba9feba31d9be8e368ca))
- Orgs ([#1720](https://github.com/shuttle-hq/shuttle/issues/1720)) - ([4c80f01](https://github.com/shuttle-hq/shuttle/commit/4c80f011fabf83c93f2bcbb1d68f92c5846e334f))
- CORS support on gateway ([#1714](https://github.com/shuttle-hq/shuttle/issues/1714)) - ([f752263](https://github.com/shuttle-hq/shuttle/commit/f752263c355d99efbbf2515d4c77ca79182d575b))

### Bug Fixes

- *(cargo-shuttle)* Deploy http method - ([cd7b71c](https://github.com/shuttle-hq/shuttle/commit/cd7b71c1c672078c7865da2be42471cc33ff06e5))
- *(runtime)* Re-export Environment ([#1736](https://github.com/shuttle-hq/shuttle/issues/1736)) - ([7c42c05](https://github.com/shuttle-hq/shuttle/commit/7c42c05c42a756f7920398e03045939b9442abed))
- Rustls and gix cargo audit failures ([#1749](https://github.com/shuttle-hq/shuttle/issues/1749)) - ([5157708](https://github.com/shuttle-hq/shuttle/commit/51577089cbedac1bc662b066a077c9b8a59dba43))
- Runtime panic ([#1735](https://github.com/shuttle-hq/shuttle/issues/1735)) - ([2286c04](https://github.com/shuttle-hq/shuttle/commit/2286c0406669f170e69c73c665428b12b5ff2558))
- Various fixes and corrections ([#1729](https://github.com/shuttle-hq/shuttle/issues/1729)) - ([1922b0e](https://github.com/shuttle-hq/shuttle/commit/1922b0ee2bb77a78504d746e91cafa493c1a55f0))

### Refactor

- Errors ([#1738](https://github.com/shuttle-hq/shuttle/issues/1738)) - ([4e7745f](https://github.com/shuttle-hq/shuttle/commit/4e7745fad6d99a0f8a091fd8e349a92ef1f0259b))
- Simplify by getting rid of the unneeded User wrapper ([#1722](https://github.com/shuttle-hq/shuttle/issues/1722)) - ([d536aa4](https://github.com/shuttle-hq/shuttle/commit/d536aa408658ca0783028670d8b67b844e06f16a))
- Only use permit to check project permissions ([#1721](https://github.com/shuttle-hq/shuttle/issues/1721)) - ([d155595](https://github.com/shuttle-hq/shuttle/commit/d155595a3c92fab475fdf25938f66b1288d1093d))

### Miscellaneous Tasks

- V0.44.0 ([#1750](https://github.com/shuttle-hq/shuttle/issues/1750)) - ([8a1b1ce](https://github.com/shuttle-hq/shuttle/commit/8a1b1cee7f060cdf9fa4bac86415770e2d15ee96))
- Bump h2 to patch continuation flood vulnerability ([#1719](https://github.com/shuttle-hq/shuttle/issues/1719)) - ([4eda51f](https://github.com/shuttle-hq/shuttle/commit/4eda51f66c43f06e344fd345e3361b95afe7b464))
- Clean up dev deps ([#1712](https://github.com/shuttle-hq/shuttle/issues/1712)) - ([7c48569](https://github.com/shuttle-hq/shuttle/commit/7c48569b671a270485d00bf93a0535845e645645))
- Set package.repository for all crates ([#1716](https://github.com/shuttle-hq/shuttle/issues/1716)) - ([0525f33](https://github.com/shuttle-hq/shuttle/commit/0525f33129f11122e502ec5b39e5ff81e24d3566))

### Miscellaneous

- Revert severity level on allowed project names ([#1739](https://github.com/shuttle-hq/shuttle/issues/1739)) - ([f8f9817](https://github.com/shuttle-hq/shuttle/commit/f8f9817dcca72063b77c24b393bf97af1456db3d))
- Added beta deployment list ([#1741](https://github.com/shuttle-hq/shuttle/issues/1741)) - ([a44d095](https://github.com/shuttle-hq/shuttle/commit/a44d095a42c7967c14cf321c3293694692807948))
- Add log streaming in beta platform ([#1743](https://github.com/shuttle-hq/shuttle/issues/1743)) - ([8767db4](https://github.com/shuttle-hq/shuttle/commit/8767db4b526f26d4dd80c152f07a58923a1ddf32))
- Added beta deployment status  ([#1740](https://github.com/shuttle-hq/shuttle/issues/1740)) - ([6231811](https://github.com/shuttle-hq/shuttle/commit/6231811da58babffe2644b13d45138e4710f499d))
- Update deploy cmd for beta platform  ([#1734](https://github.com/shuttle-hq/shuttle/issues/1734)) - ([d3f6c6b](https://github.com/shuttle-hq/shuttle/commit/d3f6c6b9d4818cc39aff1afe33a4e7af504a0193))
- Merge branch 'main' into feat/shuttle-ecs-common - ([f732be0](https://github.com/shuttle-hq/shuttle/commit/f732be0451a0e44d9d1f54a8a65e78a4d031f701))
- Nix shell ([#1726](https://github.com/shuttle-hq/shuttle/issues/1726)) - ([178e77c](https://github.com/shuttle-hq/shuttle/commit/178e77cede54ec58b9e50a37e16e68d2f2a185ff))

## [0.43.0](https://github.com/shuttle-hq/shuttle/compare/v0.42.0..v0.43.0) - 2024-04-02

### Features

- *(cargo-shuttle)* --debug ([#1689](https://github.com/shuttle-hq/shuttle/issues/1689)) - ([38510fb](https://github.com/shuttle-hq/shuttle/commit/38510fb16f4857806efeec6e753d8b2284e60c3a))
- *(cargo-shuttle)* Remove retry client, add version header ([#1691](https://github.com/shuttle-hq/shuttle/issues/1691)) - ([0aac5d7](https://github.com/shuttle-hq/shuttle/commit/0aac5d7e6531ca1f0725f1fff7d18862486014c5))
- Gateway command to sync permit ([#1705](https://github.com/shuttle-hq/shuttle/issues/1705)) - ([8a38a12](https://github.com/shuttle-hq/shuttle/commit/8a38a126e95a6c08f7b782a3503f30351e7d2a84))
- Add resource request struct to backends ([#1718](https://github.com/shuttle-hq/shuttle/issues/1718)) - ([45e5d4e](https://github.com/shuttle-hq/shuttle/commit/45e5d4ed29915d6ae43fd37a895794f5d32b37cd))
- Merge runtime updates in main ecs branch ([#1709](https://github.com/shuttle-hq/shuttle/issues/1709)) - ([e40002b](https://github.com/shuttle-hq/shuttle/commit/e40002bb9466ddbcd1cc240b7ec3a2c3aa7cc0c0))
- Generated Permit client, project permission logic ([#1699](https://github.com/shuttle-hq/shuttle/issues/1699)) - ([e33329b](https://github.com/shuttle-hq/shuttle/commit/e33329bd8c0cc09866d9399f8fc07e8e63e36031))
- Auth sync users with permit ([#1703](https://github.com/shuttle-hq/shuttle/issues/1703)) - ([e4e8e01](https://github.com/shuttle-hq/shuttle/commit/e4e8e01c7137e62f810a1750765b8bcd0d69e527))
- Permit pdp in docker stack ([#1697](https://github.com/shuttle-hq/shuttle/issues/1697)) - ([1b7a8a1](https://github.com/shuttle-hq/shuttle/commit/1b7a8a12adfee02ae78526c7a0991854ff2baa68))
- Basic Permit client with tests ([#1693](https://github.com/shuttle-hq/shuttle/issues/1693)) - ([29646a3](https://github.com/shuttle-hq/shuttle/commit/29646a3167175a75749598564355c0e642c59658))
- Permit client skeleton ([#1696](https://github.com/shuttle-hq/shuttle/issues/1696)) - ([b1e029a](https://github.com/shuttle-hq/shuttle/commit/b1e029a27ca4ea2cde6ff3d4decbb394378e8d9f))
- Update services api client ([#1695](https://github.com/shuttle-hq/shuttle/issues/1695)) - ([27c5c37](https://github.com/shuttle-hq/shuttle/commit/27c5c370883b11e794f4c6ee66e91a1622214374))

### Bug Fixes

- *(codegen)* Use full path for std types ([#1702](https://github.com/shuttle-hq/shuttle/issues/1702)) - ([71e240f](https://github.com/shuttle-hq/shuttle/commit/71e240fae0446a2beb19df79cfa369706d41b75c))
- *(logger)* Don't block when deleting old logs ([#1690](https://github.com/shuttle-hq/shuttle/issues/1690)) - ([cb9559f](https://github.com/shuttle-hq/shuttle/commit/cb9559fca269a12d9a4b5649131a948e13d60e1b))
- *(shuttle-turso)* Use open_remote when using local_addr ([#1701](https://github.com/shuttle-hq/shuttle/issues/1701)) - ([c437091](https://github.com/shuttle-hq/shuttle/commit/c4370913eae4633416cb3324f0b9b7c38a3fcb99))

### Refactor

- Shuttle-common/backend -> shuttle-backends ([#1698](https://github.com/shuttle-hq/shuttle/issues/1698)) - ([ee7809d](https://github.com/shuttle-hq/shuttle/commit/ee7809d1856226804834eaf97d6cf4ef8b10348f))

### Miscellaneous Tasks

- *(shuttle-turso)* Update libsql dep ([#1694](https://github.com/shuttle-hq/shuttle/issues/1694)) - ([bd9466f](https://github.com/shuttle-hq/shuttle/commit/bd9466fead12f38c5970ceea18b49d83f9fdc36e))
- V0.43.0 ([#1710](https://github.com/shuttle-hq/shuttle/issues/1710)) - ([32bb5af](https://github.com/shuttle-hq/shuttle/commit/32bb5af437c175e6f8d9496c43f2b15c36ac0ebd))
- Filter jobs based on if PR is from fork ([#1700](https://github.com/shuttle-hq/shuttle/issues/1700)) - ([998485d](https://github.com/shuttle-hq/shuttle/commit/998485de0e616083fc3e222407ab7fe0bc51921a))
- Remove secrets, metadata crates ([#1688](https://github.com/shuttle-hq/shuttle/issues/1688)) - ([4ab5f08](https://github.com/shuttle-hq/shuttle/commit/4ab5f083d8bd98d1136974cbd7e89558b6bf4752))
- Remove e2e crate, update development docs ([#1684](https://github.com/shuttle-hq/shuttle/issues/1684)) - ([5a5c08e](https://github.com/shuttle-hq/shuttle/commit/5a5c08e8ca44d2ab3629b805aec714fcd9f04426))

### Miscellaneous

- Merge remote-tracking branch 'upstream/main' into feat/shuttle-ecs-common - ([01b69a2](https://github.com/shuttle-hq/shuttle/commit/01b69a2c1017bc0eeb79199aeb5212ca40600e90))
- Reinstate http otel exporter - ([222b613](https://github.com/shuttle-hq/shuttle/commit/222b61394e840d5671741af45cd702cfe49dbf22))
- Add control backend type - ([f60106c](https://github.com/shuttle-hq/shuttle/commit/f60106cfbc1b85815e5ddb883e848764fa65f920))

## [0.42.0](https://github.com/shuttle-hq/shuttle/compare/v0.41.0..v0.42.0) - 2024-03-18

### Features

- *(resources)* Add `diesel-async` support for `shuttle-shared-db` ([#1664](https://github.com/shuttle-hq/shuttle/issues/1664)) - ([cd5476c](https://github.com/shuttle-hq/shuttle/commit/cd5476c112e6179cfa7c81a19e26f8d16cdc1a58))
- Fall back to finding Secrets.toml in workspace root ([#1682](https://github.com/shuttle-hq/shuttle/issues/1682)) - ([6acabb7](https://github.com/shuttle-hq/shuttle/commit/6acabb728611a983dd1fdaef859476f7eee9c26d))

### Bug Fixes

- *(cargo-shuttle)* Remove integration test example ([#1672](https://github.com/shuttle-hq/shuttle/issues/1672)) - ([05a3765](https://github.com/shuttle-hq/shuttle/commit/05a37656b272255cd665ba58fa411bb52abc86d0))
- *(deployer)* Check correct config field(s) in resource cache ([#1675](https://github.com/shuttle-hq/shuttle/issues/1675)) - ([f547060](https://github.com/shuttle-hq/shuttle/commit/f547060992819b7b51c5f51fa04ead804fdd1d0e))
- Patches script root-relative path ([#1685](https://github.com/shuttle-hq/shuttle/issues/1685)) - ([2428e99](https://github.com/shuttle-hq/shuttle/commit/2428e9919c82a1ac5762bee3ec911362699409f3))

### Refactor

- *(auth, gateway)* Use user_id over account_name ([#1674](https://github.com/shuttle-hq/shuttle/issues/1674)) - ([4937f4b](https://github.com/shuttle-hq/shuttle/commit/4937f4ba97a1d29eac093721742df47ae5e04dc3))
- Move secrets and metadata plugins to runtime ([#1673](https://github.com/shuttle-hq/shuttle/issues/1673)) - ([f15e6bb](https://github.com/shuttle-hq/shuttle/commit/f15e6bb7c4b4b195cfe09aa46d8202d12ba4bdc0))

### Miscellaneous Tasks

- V0.42.0 ([#1686](https://github.com/shuttle-hq/shuttle/issues/1686)) - ([7f2a195](https://github.com/shuttle-hq/shuttle/commit/7f2a19502eaf39c7952fed7fd40ae9175a04b26b))
- Fix unstable filter ([#1683](https://github.com/shuttle-hq/shuttle/issues/1683)) - ([dbeab20](https://github.com/shuttle-hq/shuttle/commit/dbeab20d1cae40712c6c2fcdddf917d76ed7c17a))
- Update README.md ([#1681](https://github.com/shuttle-hq/shuttle/issues/1681)) - ([27c88d3](https://github.com/shuttle-hq/shuttle/commit/27c88d3b921e508d07ff02267344002fbe948b8b))

### Miscellaneous

- *(deployer)* Improve get_logs out_of_range error ([#1676](https://github.com/shuttle-hq/shuttle/issues/1676)) - ([ef3f184](https://github.com/shuttle-hq/shuttle/commit/ef3f184d9ca15de8f3f8ef4ec1b10f294c5eacdf))
- Update README.md ([#1680](https://github.com/shuttle-hq/shuttle/issues/1680)) - ([7663cac](https://github.com/shuttle-hq/shuttle/commit/7663cac8a7b60ff2163f20f664aed475d72d50a0))
- Update README.md ([#1679](https://github.com/shuttle-hq/shuttle/issues/1679)) - ([5cfcdf7](https://github.com/shuttle-hq/shuttle/commit/5cfcdf7c5349f89ac906e1419591c646b44a0f78))
- Delete project restarts them first if oudated ([#1677](https://github.com/shuttle-hq/shuttle/issues/1677)) - ([b40a14b](https://github.com/shuttle-hq/shuttle/commit/b40a14b2ef0056a68f69806d056f17dc7255b4d3))
- Add `--raw` flag to `run`, `deploy` command ([#1653](https://github.com/shuttle-hq/shuttle/issues/1653)) - ([db4f2e6](https://github.com/shuttle-hq/shuttle/commit/db4f2e6e29eff3612bdf85687875cb137e857980))

## [0.41.0](https://github.com/shuttle-hq/shuttle/compare/v0.40.0..v0.41.0) - 2024-03-07

### Features

- *(cargo-shuttle)* New template system for init ([#1667](https://github.com/shuttle-hq/shuttle/issues/1667)) - ([7c393a8](https://github.com/shuttle-hq/shuttle/commit/7c393a8eadfeac8165219a79227b4dc1691f8bc4))
- *(install)* Change powershell installation script ([#1636](https://github.com/shuttle-hq/shuttle/issues/1636)) - ([0c57746](https://github.com/shuttle-hq/shuttle/commit/0c577463ccf5af6328a09c5ff82c1f099e6dbda5))
- Migration for user_id ([#1663](https://github.com/shuttle-hq/shuttle/issues/1663)) - ([83d9651](https://github.com/shuttle-hq/shuttle/commit/83d965134190b079bf23043d8421f41079d35153))

### Bug Fixes

- *(auth)* User query columns ([#1669](https://github.com/shuttle-hq/shuttle/issues/1669)) - ([b3f3c60](https://github.com/shuttle-hq/shuttle/commit/b3f3c6071a808e2ae5ec595cfc4283d149ec258b))
- *(common)* Public fields, v0.40.2 ([#1662](https://github.com/shuttle-hq/shuttle/issues/1662)) - ([dc74c42](https://github.com/shuttle-hq/shuttle/commit/dc74c4276f333e5fc201e82e916cc9d8b0003dc8))
- *(common)* Add missing schema, v0.40.1 ([#1661](https://github.com/shuttle-hq/shuttle/issues/1661)) - ([3e11ae3](https://github.com/shuttle-hq/shuttle/commit/3e11ae3ef74ce0294cfe3331d7a1ecb9e2b5b3e4))
- *(gateway)* Proxy wait for service port to open ([#1668](https://github.com/shuttle-hq/shuttle/issues/1668)) - ([93562fc](https://github.com/shuttle-hq/shuttle/commit/93562fc886a815743c32d8ef74f90c6343f78588))

### Miscellaneous Tasks

- V0.41.0 ([#1670](https://github.com/shuttle-hq/shuttle/issues/1670)) - ([ad834ae](https://github.com/shuttle-hq/shuttle/commit/ad834aee21c134c6f913d458bc1e23ce1b8c4eb6))
- Update test-context dependency ([#1665](https://github.com/shuttle-hq/shuttle/issues/1665)) - ([d39724a](https://github.com/shuttle-hq/shuttle/commit/d39724a6c4594c8a706c6041347eaedae6a08aea))
- Remove buildx cache, bump versions ([#1650](https://github.com/shuttle-hq/shuttle/issues/1650)) - ([f41afde](https://github.com/shuttle-hq/shuttle/commit/f41afde9065be9b9892f7ddf68dd6ff4a4b57f44))

### Miscellaneous

- Remove shuttle-next ([#1652](https://github.com/shuttle-hq/shuttle/issues/1652)) - ([98551c9](https://github.com/shuttle-hq/shuttle/commit/98551c9f3521d451c54281ca80f402c624c7356e))

## [0.40.0](https://github.com/shuttle-hq/shuttle/compare/v0.39.0..v0.40.0) - 2024-03-04

### Features

- *(cargo-shuttle)* Add project name to the default directory, ask again if path is rejected ([#1654](https://github.com/shuttle-hq/shuttle/issues/1654)) - ([e6f2b2f](https://github.com/shuttle-hq/shuttle/commit/e6f2b2fd77bdbea9e0d5f82b0a3f80012e479aa0))
- *(cargo-shuttle)* `--secrets` arg to use non-default secrets file ([#1642](https://github.com/shuttle-hq/shuttle/issues/1642)) - ([751f337](https://github.com/shuttle-hq/shuttle/commit/751f337dbd7ba410f659221de24ad0eebd9b92dd))
- *(common)* Add template definition schema ([#1655](https://github.com/shuttle-hq/shuttle/issues/1655)) - ([c386702](https://github.com/shuttle-hq/shuttle/commit/c386702a84f3e74656f287fc776b7ab1a72bd359))
- *(deployer)* Load phase caching, automatic startup ([#1640](https://github.com/shuttle-hq/shuttle/issues/1640)) - ([ada3fe1](https://github.com/shuttle-hq/shuttle/commit/ada3fe1d32884142ecb4b51c8ff4189a02ad19a3))
- *(install.sh)* Always check for cargo install first ([#1610](https://github.com/shuttle-hq/shuttle/issues/1610)) - ([cca27d9](https://github.com/shuttle-hq/shuttle/commit/cca27d953ff09b55b6f0b79d8f06d4eb8cc44385))
- *(runtime, deployer)* [**breaking**] Extract load phase + provisioning to deployer, resource update ([#1628](https://github.com/shuttle-hq/shuttle/issues/1628)) - ([ba57785](https://github.com/shuttle-hq/shuttle/commit/ba57785a6054f8f04c4e0b42730a11d368e9efa3))
- RDS custom database name ([#1651](https://github.com/shuttle-hq/shuttle/issues/1651)) - ([958399c](https://github.com/shuttle-hq/shuttle/commit/958399cde40ae4d1b4b5d9beaf455736ac5d84e0))
- Enable Datadog APM error tracking with a tracing layer ([#1626](https://github.com/shuttle-hq/shuttle/issues/1626)) - ([c5f2caf](https://github.com/shuttle-hq/shuttle/commit/c5f2caf037eb5ce05dda80dcd01ba8618319d3be))

### Bug Fixes

- *(cargo-shuttle)* Windows build ([#1648](https://github.com/shuttle-hq/shuttle/issues/1648)) - ([9614da4](https://github.com/shuttle-hq/shuttle/commit/9614da4e5f05fb62f3b6db1c56b50a6cfbcf1b67))
- *(gateway)* Custom domain followup improvements ([#1627](https://github.com/shuttle-hq/shuttle/issues/1627)) - ([46c71e7](https://github.com/shuttle-hq/shuttle/commit/46c71e7cc95f6dd018dc1160b338e30bc39d29f1))
- *(resource-recorder)* [**breaking**] Disable service id endpoint ([#1644](https://github.com/shuttle-hq/shuttle/issues/1644)) - ([0b97911](https://github.com/shuttle-hq/shuttle/commit/0b9791125774a965c25476fb05e971ac05ceb7fb))
- *(resource-recorder, provisioner)* Fix integration tests ([#1645](https://github.com/shuttle-hq/shuttle/issues/1645)) - ([b4f6577](https://github.com/shuttle-hq/shuttle/commit/b4f6577d0f1bcf36a9458da2f61979b16a28e497))
- Cargo audit ([#1657](https://github.com/shuttle-hq/shuttle/issues/1657)) - ([5742dc8](https://github.com/shuttle-hq/shuttle/commit/5742dc862feea3e6ea10dd169189511cacc8d241))
- Remove builder from compose ([#1643](https://github.com/shuttle-hq/shuttle/issues/1643)) - ([b637bef](https://github.com/shuttle-hq/shuttle/commit/b637bef9bcafc99a938107fa2c8589496653aa97))
- Various fixes ([#1641](https://github.com/shuttle-hq/shuttle/issues/1641)) - ([b026bd5](https://github.com/shuttle-hq/shuttle/commit/b026bd519dbfe5bf8f47f7670d170df5b420f5fe))

### Miscellaneous Tasks

- Use newer linux images ([#1659](https://github.com/shuttle-hq/shuttle/issues/1659)) - ([22c3695](https://github.com/shuttle-hq/shuttle/commit/22c369526b8909e4def42fa1a875493f15195c45))
- V0.40.0 ([#1646](https://github.com/shuttle-hq/shuttle/issues/1646)) - ([30f075a](https://github.com/shuttle-hq/shuttle/commit/30f075aef182f0154b5bd3d6417132967ba3ce4d))
- Remove builder ([#1637](https://github.com/shuttle-hq/shuttle/issues/1637)) - ([637b0f2](https://github.com/shuttle-hq/shuttle/commit/637b0f27c64b1ab8ff12be2d76bd9049d8a4e662))

### Miscellaneous

- *(provisioner)* Check project ownership in APIs ([#1630](https://github.com/shuttle-hq/shuttle/issues/1630)) - ([6e135a0](https://github.com/shuttle-hq/shuttle/commit/6e135a021f57d953851100d46fce16bbbc28a774))
- Make passwords longer ([#1649](https://github.com/shuttle-hq/shuttle/issues/1649)) - ([3080c93](https://github.com/shuttle-hq/shuttle/commit/3080c93d063524b29d48012a9e1484f26b0cca5c))
- Use cargo-chef 0.1.64 --bin flags ([#1638](https://github.com/shuttle-hq/shuttle/issues/1638)) - ([acddd82](https://github.com/shuttle-hq/shuttle/commit/acddd8276b37dea474ecb3dac78e1e6efe293d87))

## [0.39.0](https://github.com/shuttle-hq/shuttle/compare/v0.38.0..v0.39.0) - 2024-02-14

### Features

- *(cargo-shuttle)* Add loco to init command ([#1620](https://github.com/shuttle-hq/shuttle/issues/1620)) - ([df98061](https://github.com/shuttle-hq/shuttle/commit/df98061640da33713719429ee1137ab405d251fa))
- *(deployer, gateway)* Remove deployer proxy ([#1612](https://github.com/shuttle-hq/shuttle/issues/1612)) - ([d42cc56](https://github.com/shuttle-hq/shuttle/commit/d42cc5622b3767036f5ba5450a8e211492f71e83))
- *(gateway)* Allow multiple hostnames, proxy caching ([#1616](https://github.com/shuttle-hq/shuttle/issues/1616)) - ([2112f24](https://github.com/shuttle-hq/shuttle/commit/2112f249ca98628cb86871b27540eb7b56b9bbb5))
- *(logger)* Clean old logs on startup ([#1619](https://github.com/shuttle-hq/shuttle/issues/1619)) - ([c0d29d1](https://github.com/shuttle-hq/shuttle/commit/c0d29d1ec144e66bbd25da08d4bba3e1a81cd682))
- Add OpenDAL resource support ([#1617](https://github.com/shuttle-hq/shuttle/issues/1617)) - ([963fdab](https://github.com/shuttle-hq/shuttle/commit/963fdab9ba4809b6bc74ca73e007fc9dabac1664))
- Disable trace_layer on_failure ([#1608](https://github.com/shuttle-hq/shuttle/issues/1608)) - ([8c6e931](https://github.com/shuttle-hq/shuttle/commit/8c6e931efbb17272baeab54727815a2165c4c7ef))

### Bug Fixes

- *(gateway)* Don't try to deserialize non-200 res to service summary ([#1607](https://github.com/shuttle-hq/shuttle/issues/1607)) - ([9a92094](https://github.com/shuttle-hq/shuttle/commit/9a9209404ed7624f33bfb1ee4f66fa2c67fb98e7))
- *(shuttle-qdrant)* V0.38.0 ([#1606](https://github.com/shuttle-hq/shuttle/issues/1606)) - ([328b2f8](https://github.com/shuttle-hq/shuttle/commit/328b2f8739cc7d35a7777abe9b804bf892461e27))
- Submodule ([#1625](https://github.com/shuttle-hq/shuttle/issues/1625)) - ([3c4e6b3](https://github.com/shuttle-hq/shuttle/commit/3c4e6b38825aa26b55cb5ae063866948d8bc968c))
- Remove obsolete resources ([#1543](https://github.com/shuttle-hq/shuttle/issues/1543)) - ([61b4e1d](https://github.com/shuttle-hq/shuttle/commit/61b4e1d04f8b9aa2567fb4b13e0dbd9d388fb786))
- Alias updated field names from 0.37.0 ([#1618](https://github.com/shuttle-hq/shuttle/issues/1618)) - ([1a67ad9](https://github.com/shuttle-hq/shuttle/commit/1a67ad9ada784089925e814cef77a689b745c132))

### Refactor

- ApiError and ErrorKind to use thiserror ([#1615](https://github.com/shuttle-hq/shuttle/issues/1615)) - ([c7767cb](https://github.com/shuttle-hq/shuttle/commit/c7767cba85b7b31f564a970355110560fbcb66c6))
- Uniform client wrappers ([#1614](https://github.com/shuttle-hq/shuttle/issues/1614)) - ([5b5eb69](https://github.com/shuttle-hq/shuttle/commit/5b5eb69c7754285b348b7be4163f38bee2464547))

### Miscellaneous Tasks

- V0.39.0 ([#1623](https://github.com/shuttle-hq/shuttle/issues/1623)) - ([8595d77](https://github.com/shuttle-hq/shuttle/commit/8595d77d3e5bda73e8fd584ff7697d9c54969833))
- Rust 1.76 ([#1622](https://github.com/shuttle-hq/shuttle/issues/1622)) - ([031f3e1](https://github.com/shuttle-hq/shuttle/commit/031f3e170c47aadcd5239a383a152a394642c028))
- Bump git2 to resolve vulnerability ([#1621](https://github.com/shuttle-hq/shuttle/issues/1621)) - ([5720837](https://github.com/shuttle-hq/shuttle/commit/57208374b5691ed8945b9fbc24cc7b7abef76c5b))
- Move more tasks to cargo make ([#1613](https://github.com/shuttle-hq/shuttle/issues/1613)) - ([e29cb41](https://github.com/shuttle-hq/shuttle/commit/e29cb4173e54bbac6b11a507823af4fcf0ed45d3))
- Add cargo-make tasks for some maintenance and ci tasks ([#1595](https://github.com/shuttle-hq/shuttle/issues/1595)) - ([a51c70c](https://github.com/shuttle-hq/shuttle/commit/a51c70c8ea607d7c3c2291bf12396a168dc97ba0))

## [0.38.0](https://github.com/shuttle-hq/shuttle/compare/v0.37.0..v0.38.0) - 2024-02-01

### Features

- *(shuttle-turso)* [**breaking**] Replace `libsql-client` crate with the new and improved `libsql` crate ([#1531](https://github.com/shuttle-hq/shuttle/issues/1531)) - ([bd895d9](https://github.com/shuttle-hq/shuttle/commit/bd895d9d640b83303d8645bdfa7cc226165589e0))
- Have `auth` handle new subscriptions ([#1597](https://github.com/shuttle-hq/shuttle/issues/1597)) - ([bcd0966](https://github.com/shuttle-hq/shuttle/commit/bcd096604f42171d16854543295301d6d58503e9))
- Limit concurrent active cch23 projects ([#1589](https://github.com/shuttle-hq/shuttle/issues/1589)) - ([c5ddea0](https://github.com/shuttle-hq/shuttle/commit/c5ddea00829ff2a3c75676c97a52ef684e12a255))
- Write a wrapper for the provisioner to call gw and r-r clients ([#1585](https://github.com/shuttle-hq/shuttle/issues/1585)) - ([4a2ab9e](https://github.com/shuttle-hq/shuttle/commit/4a2ab9ef75e9147cddffc4ea1570a3a34753321f))

### Bug Fixes

- *(cargo-shuttle)* Set name in Shuttle.toml when cloning a workspace ([#1599](https://github.com/shuttle-hq/shuttle/issues/1599)) - ([e0e94c9](https://github.com/shuttle-hq/shuttle/commit/e0e94c920b6ba5b09351524326b911f4268ee51d))
- *(gateway)* Rename scope field to shuttle.project.name ([#1538](https://github.com/shuttle-hq/shuttle/issues/1538)) - ([c25c187](https://github.com/shuttle-hq/shuttle/commit/c25c18730e259618cf1f74d1dc7b3f3e64ecf1a7))
- *(otel)* Increase trace_buffer for dd exporter ([#1587](https://github.com/shuttle-hq/shuttle/issues/1587)) - ([50e6a4c](https://github.com/shuttle-hq/shuttle/commit/50e6a4cc7002f025d81177fa14e1305920e92bbb))
- Qdrant docker image name ([#1539](https://github.com/shuttle-hq/shuttle/issues/1539)) - ([41f9159](https://github.com/shuttle-hq/shuttle/commit/41f9159387cefc3d87acd161c5a63a7a8cdfc0aa))
- Remove common_tests dep from common ([#1537](https://github.com/shuttle-hq/shuttle/issues/1537)) - ([66d5d9b](https://github.com/shuttle-hq/shuttle/commit/66d5d9b34f0969834899575a88335ad3b2a4b4be))

### Documentation

- *(shuttle-turso)* Update docs for turso libsql change ([#1591](https://github.com/shuttle-hq/shuttle/issues/1591)) - ([fcec01f](https://github.com/shuttle-hq/shuttle/commit/fcec01f509c3510f504c86d82cc5dfc4c1633be9))
- Update README ([#1594](https://github.com/shuttle-hq/shuttle/issues/1594)) - ([69864f8](https://github.com/shuttle-hq/shuttle/commit/69864f8cd4628452c312b4b5f56b41e9f3549f3b))

### Miscellaneous Tasks

- V0.38.0 ([#1598](https://github.com/shuttle-hq/shuttle/issues/1598)) - ([c9b916e](https://github.com/shuttle-hq/shuttle/commit/c9b916e8113e6046c90d654a2176c5feb0e6bec9))
- Disable unused builder service until it is needed ([#1542](https://github.com/shuttle-hq/shuttle/issues/1542)) - ([f210c8f](https://github.com/shuttle-hq/shuttle/commit/f210c8fbfa4fde2546b3142d4c4a7463f8195ada))
- Remove utoipa openAPI docs ([#1588](https://github.com/shuttle-hq/shuttle/issues/1588)) - ([ece23c7](https://github.com/shuttle-hq/shuttle/commit/ece23c7e7af8bfbde9f2b5932054f5189d6b30ec))
- Remove panamax and deck-chores ([#1540](https://github.com/shuttle-hq/shuttle/issues/1540)) - ([ed81509](https://github.com/shuttle-hq/shuttle/commit/ed8150915118cafc4e712016670a39b057efcf4d))
- Remove shuttle-poise ([#1541](https://github.com/shuttle-hq/shuttle/issues/1541)) - ([c5e2d4b](https://github.com/shuttle-hq/shuttle/commit/c5e2d4b2ff8e4e71c038a35ff8fe049113465d72))

### Miscellaneous

- Remove session ([#1596](https://github.com/shuttle-hq/shuttle/issues/1596)) - ([4dd7971](https://github.com/shuttle-hq/shuttle/commit/4dd7971816bb5b2482c2b7bc61745e7de55bb925))
- Add operation_name field for task polling ([#1590](https://github.com/shuttle-hq/shuttle/issues/1590)) - ([625a015](https://github.com/shuttle-hq/shuttle/commit/625a0151e68a1e70361aedd7a659a9de0b1f5fc2))
- Small nitpicks ([#1544](https://github.com/shuttle-hq/shuttle/issues/1544)) - ([9a75f16](https://github.com/shuttle-hq/shuttle/commit/9a75f1640e177a8eda81318e44a89e7ba3e30e3a))

## [0.37.0](https://github.com/shuttle-hq/shuttle/compare/v0.36.0..v0.37.0) - 2024-01-23

### Features

- *(auth)* Add subscriptions table to auth, add rds quota to claim limits ([#1529](https://github.com/shuttle-hq/shuttle/issues/1529)) - ([02d68a5](https://github.com/shuttle-hq/shuttle/commit/02d68a5227c3ad6affff21202222c1a15148357c))
- *(resources)* [**breaking**] Get db connection string from resources, refactor ResourceBuilder ([#1522](https://github.com/shuttle-hq/shuttle/issues/1522)) - ([c6eae64](https://github.com/shuttle-hq/shuttle/commit/c6eae64ceb1fa262404d5d60b0f9c9a83b0b641d))
- *(shuttle-poem)* Support poem v2.0.0 ([#1520](https://github.com/shuttle-hq/shuttle/issues/1520)) - ([cf37eb5](https://github.com/shuttle-hq/shuttle/commit/cf37eb56651c78d417a843da65be7b5d4db9dc1b))
- *(shuttle-serenity)* Make serenity 0.12 default, support poise 0.6, deprecate shuttle-poise ([#1521](https://github.com/shuttle-hq/shuttle/issues/1521)) - ([d6e6a46](https://github.com/shuttle-hq/shuttle/commit/d6e6a463bad4957d0119a858a090707bdf2e6336))
- Qdrant resource ([#1025](https://github.com/shuttle-hq/shuttle/issues/1025)) - ([20c1251](https://github.com/shuttle-hq/shuttle/commit/20c1251491e90fd388b34df5402c66f1218643bc))
- Base api and gateway client ([#1525](https://github.com/shuttle-hq/shuttle/issues/1525)) - ([fb38ccc](https://github.com/shuttle-hq/shuttle/commit/fb38cccda021a620d018d50ac83c904c1fd0e5d6))
- Add version tag to our deployment ([#1528](https://github.com/shuttle-hq/shuttle/issues/1528)) - ([a7c2f6e](https://github.com/shuttle-hq/shuttle/commit/a7c2f6e3c5afdb672e0ac68b81a9c7ab152465bc))

### Bug Fixes

- *(gateway)* Uppercase old ulids ([#1424](https://github.com/shuttle-hq/shuttle/issues/1424)) - ([f23703e](https://github.com/shuttle-hq/shuttle/commit/f23703e09eb919ef4e12d684d4f16099ccb07650))
- *(proxy)* Record http.host after parsing to avoid Some(..) in the attr ([#1527](https://github.com/shuttle-hq/shuttle/issues/1527)) - ([68b2110](https://github.com/shuttle-hq/shuttle/commit/68b21106c5a3d27f41b61b0b0d34b9fd3ea5b532))

### Refactor

- *(gateway)* Renew gateway certificate returns more info about success ([#1492](https://github.com/shuttle-hq/shuttle/issues/1492)) - ([4ffc8de](https://github.com/shuttle-hq/shuttle/commit/4ffc8dee645c296879ac068c28d2cc10e788500e))

### Testing

- *(auth)* Simplify auth service tests with wiremock ([#1514](https://github.com/shuttle-hq/shuttle/issues/1514)) - ([129c329](https://github.com/shuttle-hq/shuttle/commit/129c329737feff91943a4f4d29b0ca58615d18ad))

### Miscellaneous Tasks

- V0.37.0 ([#1535](https://github.com/shuttle-hq/shuttle/issues/1535)) - ([9c1199c](https://github.com/shuttle-hq/shuttle/commit/9c1199c950f24804d23eb8ff631d92d3b289d667))
- Update wiremock to resolve cargo audit ([#1534](https://github.com/shuttle-hq/shuttle/issues/1534)) - ([43e0c12](https://github.com/shuttle-hq/shuttle/commit/43e0c12f3b01c555d8ee80f2114172c56e5ebbdb))
- Use default ubuntu image ([#1524](https://github.com/shuttle-hq/shuttle/issues/1524)) - ([d9ad017](https://github.com/shuttle-hq/shuttle/commit/d9ad0179a5d3b3ff0194b81c1558e181addcf592))
- Upgrade h2 to resolve cargo audit ([#1530](https://github.com/shuttle-hq/shuttle/issues/1530)) - ([84c52c5](https://github.com/shuttle-hq/shuttle/commit/84c52c59a9a6c92879ad1fd06e3ede6d97d0cb86))
- Split release flow ([#1518](https://github.com/shuttle-hq/shuttle/issues/1518)) - ([8c45aea](https://github.com/shuttle-hq/shuttle/commit/8c45aeaddd6945d803f360c2719fcf1aec4e6f53))
- Don't fail release flow if already published - ([fd5f20d](https://github.com/shuttle-hq/shuttle/commit/fd5f20d17618f34e531ebfa525794e31687b65f4))

### Miscellaneous

- Improve shuttle-runtime out-of-date hint ([#1533](https://github.com/shuttle-hq/shuttle/issues/1533)) - ([be2198c](https://github.com/shuttle-hq/shuttle/commit/be2198cd75f6636d7faa68d17b77c84265225759))
- Set shuttle.project.name in more places ([#1526](https://github.com/shuttle-hq/shuttle/issues/1526)) - ([ab179c3](https://github.com/shuttle-hq/shuttle/commit/ab179c3c6a5d91b4d5697ab8aa366918657def29))

## [0.36.0](https://github.com/shuttle-hq/shuttle/compare/v0.35.1..v0.36.0) - 2024-01-09

### Features

- *(installer)* Add windows installer script ([#1503](https://github.com/shuttle-hq/shuttle/issues/1503)) - ([52ca24a](https://github.com/shuttle-hq/shuttle/commit/52ca24a63279b3879db14886134c1964e7e3e715))
- *(service)* Emit trace with shuttle dependencies ([#1498](https://github.com/shuttle-hq/shuttle/issues/1498)) - ([90dfb72](https://github.com/shuttle-hq/shuttle/commit/90dfb72d3ac6b6dfdf3e5572f13c9beebabe6da5))
- Track project deployments ([#1508](https://github.com/shuttle-hq/shuttle/issues/1508)) - ([82f815b](https://github.com/shuttle-hq/shuttle/commit/82f815b64e41b06c2770c276c1fed7f623619055))
- `--no-git` to `cargo shuttle init` ([#1501](https://github.com/shuttle-hq/shuttle/issues/1501)) - ([05c5e53](https://github.com/shuttle-hq/shuttle/commit/05c5e5322a5825ecce55d746936f332fb7c0e287))
- Add subscription items endpoint and call it when provisioning rds ([#1478](https://github.com/shuttle-hq/shuttle/issues/1478)) - ([657815d](https://github.com/shuttle-hq/shuttle/commit/657815d07ba169cc90094b6121b3c6dc9d7d1e36))

### Bug Fixes

- *(deployer)* Return empty list when when no service is found ([#1495](https://github.com/shuttle-hq/shuttle/issues/1495)) - ([386c9cd](https://github.com/shuttle-hq/shuttle/commit/386c9cd4cfb9738848a56191e4490419f65138ee))
- *(gateway)* Dynamically pick docker stats source ([#1476](https://github.com/shuttle-hq/shuttle/issues/1476)) - ([402e3f0](https://github.com/shuttle-hq/shuttle/commit/402e3f0ba527b891360e621e68688537d9ab4ec8))
- *(provisioner)* Only delete new rds on failed subscription update ([#1488](https://github.com/shuttle-hq/shuttle/issues/1488)) - ([f81b5ef](https://github.com/shuttle-hq/shuttle/commit/f81b5efbbf167c674ae5afc9179715f6bd4af9bd))
- Tracing fixes, use consistent key names for project and service names ([#1500](https://github.com/shuttle-hq/shuttle/issues/1500)) - ([1568b1c](https://github.com/shuttle-hq/shuttle/commit/1568b1cf795686e4423ce3109d4aeeb406e9257e))

### Documentation

- Update README.md ([#1505](https://github.com/shuttle-hq/shuttle/issues/1505)) - ([7cce28b](https://github.com/shuttle-hq/shuttle/commit/7cce28b0e3543ae6f09ed3141739ce7c8a253f74))

### Miscellaneous Tasks

- *(shuttle-axum)* Use axum 0.7 by default ([#1507](https://github.com/shuttle-hq/shuttle/issues/1507)) - ([1325b12](https://github.com/shuttle-hq/shuttle/commit/1325b1208cb57592acb8626a29f69b29717c2e8d))
- *(shuttle-salvo)* Bump salvo version ([#1486](https://github.com/shuttle-hq/shuttle/issues/1486)) - ([eb7362c](https://github.com/shuttle-hq/shuttle/commit/eb7362cf49879fad204a85a6483e3f255b96c306))
- V0.36.0 ([#1511](https://github.com/shuttle-hq/shuttle/issues/1511)) - ([ad0f044](https://github.com/shuttle-hq/shuttle/commit/ad0f0440be272de9d5b9114ae2beea79477723c3))
- Rust 1.75 ([#1506](https://github.com/shuttle-hq/shuttle/issues/1506)) - ([74fb4fb](https://github.com/shuttle-hq/shuttle/commit/74fb4fba8a2815ea3ee223607efe92cf9ccb5cea))
- Guard the `/auth/key` endpoint ([#1487](https://github.com/shuttle-hq/shuttle/issues/1487)) - ([e8bb1a0](https://github.com/shuttle-hq/shuttle/commit/e8bb1a0b4fed9a1b1bc1f94f6071dd12783541b8))
- Bump zerocopy ([#1489](https://github.com/shuttle-hq/shuttle/issues/1489)) - ([ab6ab8e](https://github.com/shuttle-hq/shuttle/commit/ab6ab8e195d230d7dc8993abfbcb21e64213774a))
- Upgrade proto-gen to 0.2.0 ([#1482](https://github.com/shuttle-hq/shuttle/issues/1482)) - ([2ee5934](https://github.com/shuttle-hq/shuttle/commit/2ee59348a3124cef2e317f6bc54182f10a4d7dea))

### Revert

- Initial implementation of rds billing ([#1510](https://github.com/shuttle-hq/shuttle/issues/1510)) - ([3adff9e](https://github.com/shuttle-hq/shuttle/commit/3adff9e1edf4f8200425837e2271cc46cabc20c8))

### Miscellaneous

- Missing gateway key when trying to get jwt ([#1499](https://github.com/shuttle-hq/shuttle/issues/1499)) - ([68c8255](https://github.com/shuttle-hq/shuttle/commit/68c82555501e011b49920875a645e9f112ebe82d))

## [0.35.1](https://github.com/shuttle-hq/shuttle/compare/v0.35.0..v0.35.1) - 2023-12-13

### Features

- *(gateway)* More descriptive project not found error ([#1452](https://github.com/shuttle-hq/shuttle/issues/1452)) - ([850368e](https://github.com/shuttle-hq/shuttle/commit/850368e41c2d35f3ae2fbe0ec417d38b80636f3e))

### Bug Fixes

- *(circleci)* Missed escaping new line in deploying cmd - ([3316944](https://github.com/shuttle-hq/shuttle/commit/3316944e60f3ea1c8d8a338f985aa535d129c3e7))
- Cargo audit failures, ignore rsa advisory ([#1475](https://github.com/shuttle-hq/shuttle/issues/1475)) - ([f558b40](https://github.com/shuttle-hq/shuttle/commit/f558b40979f4341b37188d834e9457d70370df8c))

### Refactor

- Remove generics ([#1460](https://github.com/shuttle-hq/shuttle/issues/1460)) - ([84956ee](https://github.com/shuttle-hq/shuttle/commit/84956ee37ca9c93144debe0b3c25cab0df0b1630))

### Testing

- *(resource-recorder)* Allow server time to start ([#1477](https://github.com/shuttle-hq/shuttle/issues/1477)) - ([2515c23](https://github.com/shuttle-hq/shuttle/commit/2515c23d89bcf3cee45418c56f2f3d028da3dfa9))

### Miscellaneous Tasks

- V0.35.1 ([#1480](https://github.com/shuttle-hq/shuttle/issues/1480)) - ([fa0fce6](https://github.com/shuttle-hq/shuttle/commit/fa0fce6eb44218551ef71e7fe3a2fa67b03009f1))
- Store state in postgres instance ([#1420](https://github.com/shuttle-hq/shuttle/issues/1420)) - ([da538ac](https://github.com/shuttle-hq/shuttle/commit/da538ac3f3647f6b8889bd7bff037c6f35fa6f5d))

## [0.35.0](https://github.com/shuttle-hq/shuttle/compare/v0.34.1..v0.35.0) - 2023-12-07

### Features

- *(cargo-shuttle)* Change no_confirmation flag to -y/--yes, add it to resource delete ([#1470](https://github.com/shuttle-hq/shuttle/issues/1470)) - ([cc1bff0](https://github.com/shuttle-hq/shuttle/commit/cc1bff055238610bfdb66fad9cfe7cdc3f6ebfb6))
- *(cargo-shuttle)* Generate manpage ([#1388](https://github.com/shuttle-hq/shuttle/issues/1388)) - ([9bf94e8](https://github.com/shuttle-hq/shuttle/commit/9bf94e88ad29299b74e70b511aa6a4a427a1a2e7))
- *(cargo-shuttle)* Add --no-confirmation flag to project deletion ([#1468](https://github.com/shuttle-hq/shuttle/issues/1468)) - ([3e4e40b](https://github.com/shuttle-hq/shuttle/commit/3e4e40b6b4fa6af63381685a930b35470c39b7c6))
- *(gateway)* Get stats from cgroup file directly ([#1464](https://github.com/shuttle-hq/shuttle/issues/1464)) - ([564ea0b](https://github.com/shuttle-hq/shuttle/commit/564ea0b249ffd792aa2ce060d3471dc0182f2f9b))
- *(gateway)* Add back retry logic on project error ([#1455](https://github.com/shuttle-hq/shuttle/issues/1455)) - ([fda1b15](https://github.com/shuttle-hq/shuttle/commit/fda1b150a8eeefbd35d6032bf98f7a2af79fd097))
- *(gateway)* Propagate ambulance traces ([#1456](https://github.com/shuttle-hq/shuttle/issues/1456)) - ([23ba41b](https://github.com/shuttle-hq/shuttle/commit/23ba41b197a8a8d1ff8a2d80494577cf601501bf))
- *(gateway)* Add cch projects idle admin endpoint ([#1454](https://github.com/shuttle-hq/shuttle/issues/1454)) - ([e9b00db](https://github.com/shuttle-hq/shuttle/commit/e9b00db1ad9f2232672b37c046e4ce20e064caa7))
- *(gateway)* Allow manual blocking of cch project traffic at high load ([#1446](https://github.com/shuttle-hq/shuttle/issues/1446)) - ([374320d](https://github.com/shuttle-hq/shuttle/commit/374320d2d298bc456457f953a81f793a1dab370e))
- Downgrade user to basic tier only after period end ([#1427](https://github.com/shuttle-hq/shuttle/issues/1427)) - ([ad36009](https://github.com/shuttle-hq/shuttle/commit/ad360096754d2108851a2ee675707b0071464dba))
- Don't overload docker with requests ([#1457](https://github.com/shuttle-hq/shuttle/issues/1457)) - ([5c30f2b](https://github.com/shuttle-hq/shuttle/commit/5c30f2b5f0e0168da55ae98f90ce5625b36dd786))
- Protect pro tier projects and our services ([#1445](https://github.com/shuttle-hq/shuttle/issues/1445)) - ([d320d15](https://github.com/shuttle-hq/shuttle/commit/d320d15ca25d2d8c0bb5a9b50aa0ffae2b7adb64))
- Protect ourselves from going over the 1k limit ([#1444](https://github.com/shuttle-hq/shuttle/issues/1444)) - ([918eda2](https://github.com/shuttle-hq/shuttle/commit/918eda20b61fc434b7b3dd068462815dec3697f4))

### Bug Fixes

- *(cargo-shuttle)* Remove newline from errored project state output ([#1466](https://github.com/shuttle-hq/shuttle/issues/1466)) - ([b64a97f](https://github.com/shuttle-hq/shuttle/commit/b64a97fd529381ea894f202045d671ded2f20342))
- *(gateway)* Don't attempt to retry error infinitely ([#1450](https://github.com/shuttle-hq/shuttle/issues/1450)) - ([5f58d28](https://github.com/shuttle-hq/shuttle/commit/5f58d283b00eba0ea6625d97c8590ed0167e47a5))
- *(optl)* Correctly set deployment.environment resource ([#1467](https://github.com/shuttle-hq/shuttle/issues/1467)) - ([ee634a2](https://github.com/shuttle-hq/shuttle/commit/ee634a2d905c9f132881df77bd0c7fd4865ccf7a))

### Refactor

- *(gateway)* Allow stats to change in the future ([#1463](https://github.com/shuttle-hq/shuttle/issues/1463)) - ([187acc5](https://github.com/shuttle-hq/shuttle/commit/187acc5f5d16f82ae93362fd5f32c6efb3c768c4))
- *(gateway)* Only ambulance ready projects ([#1459](https://github.com/shuttle-hq/shuttle/issues/1459)) - ([c0c4e14](https://github.com/shuttle-hq/shuttle/commit/c0c4e14c4043252c6ce350933ef911a9c84e170c))
- Reduce backoff ([#1458](https://github.com/shuttle-hq/shuttle/issues/1458)) - ([60138d9](https://github.com/shuttle-hq/shuttle/commit/60138d951c13c2ef783a9217e46d8c877fb5ab0e))
- Improve build queue messages and increase queueing time ([#1447](https://github.com/shuttle-hq/shuttle/issues/1447)) - ([e822bd4](https://github.com/shuttle-hq/shuttle/commit/e822bd438383697cd77a7831a029c5b85e2dde3e))

### Documentation

- *(services)* Use readme in doc comments ([#1425](https://github.com/shuttle-hq/shuttle/issues/1425)) - ([207a63c](https://github.com/shuttle-hq/shuttle/commit/207a63cfeb7576773323cd2b1bf08abf496441b6))

### Testing

- Add an extra property claim test for pro users ([#1448](https://github.com/shuttle-hq/shuttle/issues/1448)) - ([231ec01](https://github.com/shuttle-hq/shuttle/commit/231ec017832a0a131a6f1e38aac5278291bdaeaa))

### Miscellaneous Tasks

- Fix deploy-images conditional ([#1473](https://github.com/shuttle-hq/shuttle/issues/1473)) - ([501e6c8](https://github.com/shuttle-hq/shuttle/commit/501e6c8630238a36199782d2fd4ebabfd9521396))
- V0.35.0 ([#1471](https://github.com/shuttle-hq/shuttle/issues/1471)) - ([252bdc9](https://github.com/shuttle-hq/shuttle/commit/252bdc949dd405a68b972df3c480248536d881d7))
- Fix tests with axum and serenity feature flags ([#1453](https://github.com/shuttle-hq/shuttle/issues/1453)) - ([ff1f5b2](https://github.com/shuttle-hq/shuttle/commit/ff1f5b2d7f93fbff2c43b12dcfd06e1a30e18cc2))
- Setup datadog ([#1462](https://github.com/shuttle-hq/shuttle/issues/1462)) - ([a03d051](https://github.com/shuttle-hq/shuttle/commit/a03d05197f6eb471c677d98d9380c138c2fc7b1e))

## [0.34.1](https://github.com/shuttle-hq/shuttle/compare/v0.34.0..v0.34.1) - 2023-11-29

### Features

- *(cargo-shuttle)* --raw flag on logs ([#1422](https://github.com/shuttle-hq/shuttle/issues/1422)) - ([d700cb7](https://github.com/shuttle-hq/shuttle/commit/d700cb74c02211bf4fe8d2ec7c88d5ca7edad948))
- *(gateway)* Use max 75% of cores for building ([#1434](https://github.com/shuttle-hq/shuttle/issues/1434)) - ([49bd34c](https://github.com/shuttle-hq/shuttle/commit/49bd34c934599184d74f731e9051141a8dc45b2d))
- *(gateway)* Override idle timer for cch projects ([#1430](https://github.com/shuttle-hq/shuttle/issues/1430)) - ([459426b](https://github.com/shuttle-hq/shuttle/commit/459426b1177bf5d2364bff639152694ffcf1c1b4))
- *(shuttle-axum)* Support axum 0.7 through feature flag ([#1440](https://github.com/shuttle-hq/shuttle/issues/1440)) - ([2128794](https://github.com/shuttle-hq/shuttle/commit/21287949eb529fbfb7cb27c95161205f38cf0d8c))

### Bug Fixes

- *(cargo-shuttle)* Increase runtime version check timeout ([#1437](https://github.com/shuttle-hq/shuttle/issues/1437)) - ([c4ba5a9](https://github.com/shuttle-hq/shuttle/commit/c4ba5a9b914ba8ed0998b4724f33be73e4925960))
- *(cargo-shuttle)* Handle log stream errors in deploy command ([#1429](https://github.com/shuttle-hq/shuttle/issues/1429)) - ([6d111c5](https://github.com/shuttle-hq/shuttle/commit/6d111c5da05875c227210bc5d3d2a99d58579bdb))
- *(deployer)* New secrets override old ones ([#1423](https://github.com/shuttle-hq/shuttle/issues/1423)) - ([f017db2](https://github.com/shuttle-hq/shuttle/commit/f017db2a272cf4b5bb7e92b7615478b0c990289d))
- *(shuttle-serenity)* Support serenity 0.12 through feature flag ([#1439](https://github.com/shuttle-hq/shuttle/issues/1439)) - ([0c03da0](https://github.com/shuttle-hq/shuttle/commit/0c03da0bbeb58eb3efc074eb9b2f844913370c56))
- Project delete prompt ([#1442](https://github.com/shuttle-hq/shuttle/issues/1442)) - ([e845ed0](https://github.com/shuttle-hq/shuttle/commit/e845ed08e76728addfeee563398f37fadc5b6753))

### Refactor

- Delete errored projects ([#1428](https://github.com/shuttle-hq/shuttle/issues/1428)) - ([4ab7bcf](https://github.com/shuttle-hq/shuttle/commit/4ab7bcf6e46944aba1d777e8b3c8c4b0102397a5))

### Testing

- Confirm that stopped projects delete successfully ([#1435](https://github.com/shuttle-hq/shuttle/issues/1435)) - ([b5d9b21](https://github.com/shuttle-hq/shuttle/commit/b5d9b21ec0250f8ac62b4195aa1d135910b45be3))

### Miscellaneous Tasks

- V0.34.1, cargo update, changelog ([#1433](https://github.com/shuttle-hq/shuttle/issues/1433)) - ([d213372](https://github.com/shuttle-hq/shuttle/commit/d21337222576ffe247693c641891834e6b3dd1e6))
- Bump rust versions ([#1431](https://github.com/shuttle-hq/shuttle/issues/1431)) - ([e2c63bf](https://github.com/shuttle-hq/shuttle/commit/e2c63bfff52a5517c56bc06a92e857331ad483df))

### Revert

- Rate limit based on peer address #1351 ([#1426](https://github.com/shuttle-hq/shuttle/issues/1426)) - ([bc25873](https://github.com/shuttle-hq/shuttle/commit/bc258734688b01b849ffc7f3353fef0b0a1076e5))

### Miscellaneous

- Don't do anything on delete dry run ([#1432](https://github.com/shuttle-hq/shuttle/issues/1432)) - ([312fc8f](https://github.com/shuttle-hq/shuttle/commit/312fc8f9dc7e8e9ce9ff8e148e0b1a739cc970e2))
- Remove project resources automatically when delete the project ([#1421](https://github.com/shuttle-hq/shuttle/issues/1421)) - ([5f44ea9](https://github.com/shuttle-hq/shuttle/commit/5f44ea994a3bfdb49c32b148b698736bb67bf0ef))

## [0.34.0](https://github.com/shuttle-hq/shuttle/compare/v0.33.0..v0.34.0) - 2023-11-23

### Features

- *(cargo-shuttle)* Better handling of runtime version checks ([#1418](https://github.com/shuttle-hq/shuttle/issues/1418)) - ([c677290](https://github.com/shuttle-hq/shuttle/commit/c677290887d29a3c3968ceff4fff942fa2c8168e))
- *(logger)* Rate limit based on peer address ([#1351](https://github.com/shuttle-hq/shuttle/issues/1351)) - ([4a99d4a](https://github.com/shuttle-hq/shuttle/commit/4a99d4a9351e7e557759a9300c2178f47c68d746))
- *(shuttle-serenity)* Support serenity 0.11 and 0.12, optional native tls ([#1416](https://github.com/shuttle-hq/shuttle/issues/1416)) - ([52c06a7](https://github.com/shuttle-hq/shuttle/commit/52c06a7ab9c17398ec8ff5328c99779736e3f819))
- Limit rds access to pro users ([#1398](https://github.com/shuttle-hq/shuttle/issues/1398)) - ([30b6465](https://github.com/shuttle-hq/shuttle/commit/30b6465be9ff55f54a2b0d9f4b33e81d3401a60c))
- feat(cargo-shuttle): ability to force a name to be used in init ([#1410](https://github.com/shuttle-hq/shuttle/issues/1410)) - ([8e6deae](https://github.com/shuttle-hq/shuttle/commit/8e6deaea60ffc2cba3d4ba136ef095c5fb351e58))
- Remove panamax registry override from deployers ([#1399](https://github.com/shuttle-hq/shuttle/issues/1399)) - ([4798777](https://github.com/shuttle-hq/shuttle/commit/4798777273ae29435a3a8cb05dfe08dfeabf45ac))

### Bug Fixes

- *(auth)* Comment healthcheck `start_period` & `start_interval` ([#1414](https://github.com/shuttle-hq/shuttle/issues/1414)) - ([4568805](https://github.com/shuttle-hq/shuttle/commit/45688054408b4a95ead8bfae365f063060ef2cb2))
- *(cargo-shuttle)* Cargo shuttle clean response type ([#1409](https://github.com/shuttle-hq/shuttle/issues/1409)) - ([9b1ef53](https://github.com/shuttle-hq/shuttle/commit/9b1ef53c988f268ed48857a8f54f5e8d8341691d))
- *(common)* Type conversion from str for a custom resource ([#1415](https://github.com/shuttle-hq/shuttle/issues/1415)) - ([da6b887](https://github.com/shuttle-hq/shuttle/commit/da6b8873624530e81033ea2068854610aa3b275f))
- *(gateway)* Handle invalid project names in ScopedUser ([#1396](https://github.com/shuttle-hq/shuttle/issues/1396)) - ([e9ec21b](https://github.com/shuttle-hq/shuttle/commit/e9ec21b99a9043d78d559a3844e89ca6d1fbbe7c))
- Better error hints & formatting + nits ([#1412](https://github.com/shuttle-hq/shuttle/issues/1412)) - ([2afaa16](https://github.com/shuttle-hq/shuttle/commit/2afaa16b60873878d20a513e64639732e6df12e9))
- Merge new&old secrets in deployer ([#1407](https://github.com/shuttle-hq/shuttle/issues/1407)) - ([5f5501a](https://github.com/shuttle-hq/shuttle/commit/5f5501af3fb4042f6c12a5c25a6e37329a942aab))

### Refactor

- Better feature scoping, fix turso compilation, prune library dependency tree ([#1405](https://github.com/shuttle-hq/shuttle/issues/1405)) - ([569e831](https://github.com/shuttle-hq/shuttle/commit/569e8317d7b51fc2333153a05a356573c4a6a8de))

### Documentation

- *(readme)* Update the alt text for Twitter page ([#1404](https://github.com/shuttle-hq/shuttle/issues/1404)) - ([8000d1e](https://github.com/shuttle-hq/shuttle/commit/8000d1eb2fb59cb96bd1c408e2fc3356eaa8a77c))

### Miscellaneous Tasks

- Examples v0.34.0 - ([6f16768](https://github.com/shuttle-hq/shuttle/commit/6f167685e5760584b97f6034640d833874cdb192))
- Use a centrally stored jwt signing private key ([#1402](https://github.com/shuttle-hq/shuttle/issues/1402)) - ([b7471ac](https://github.com/shuttle-hq/shuttle/commit/b7471ac2d135c9a681462775b481d55faceb18af))
- V0.34.0 ([#1417](https://github.com/shuttle-hq/shuttle/issues/1417)) - ([4e7dd2d](https://github.com/shuttle-hq/shuttle/commit/4e7dd2d49c7c847d77727ef156b52ebdff6d357c))
- Bump base64 dependency to 0.21.5 ([#1403](https://github.com/shuttle-hq/shuttle/issues/1403)) - ([263fb0d](https://github.com/shuttle-hq/shuttle/commit/263fb0d0cf2722885b18f78f51ef3d98026b3864))
- Rust 1.74 ([#1411](https://github.com/shuttle-hq/shuttle/issues/1411)) - ([b32475f](https://github.com/shuttle-hq/shuttle/commit/b32475ffce54b762f57d032130daa4819e49ddf1))
- Cargo update ([#1391](https://github.com/shuttle-hq/shuttle/issues/1391)) - ([d9c015c](https://github.com/shuttle-hq/shuttle/commit/d9c015c9e89032ab3b8ad1fe6529b32468596a57))

### Miscellaneous

- *(auth)* Added service healthcheck ([#1394](https://github.com/shuttle-hq/shuttle/issues/1394)) - ([44dfa9b](https://github.com/shuttle-hq/shuttle/commit/44dfa9b08048897a9de5f025b672bbda3eee6479))
- Delete a project even if the current state is destroyed ([#1413](https://github.com/shuttle-hq/shuttle/issues/1413)) - ([f37a0e8](https://github.com/shuttle-hq/shuttle/commit/f37a0e87c06938f0109e718c78085cb4c9267173))
- Rocket-0.5.0 stable ([#1401](https://github.com/shuttle-hq/shuttle/issues/1401)) - ([c88f0bc](https://github.com/shuttle-hq/shuttle/commit/c88f0bc9cc7ebcc56e2ecbaea30142e8d6e8ee35))

## [0.33.0](https://github.com/shuttle-hq/shuttle/compare/v0.32.0..v0.33.0) - 2023-11-16

### Features

- *(cargo-shuttle)* Make command-line aliases visible ([#1384](https://github.com/shuttle-hq/shuttle/issues/1384)) - ([434ddbf](https://github.com/shuttle-hq/shuttle/commit/434ddbf64b6b7311a84752c64b8b5d7ca7210773))
- *(gateway)* Temporary conditional project limit increase ([#1383](https://github.com/shuttle-hq/shuttle/issues/1383)) - ([4311907](https://github.com/shuttle-hq/shuttle/commit/4311907b5ba6fe5bbb1f422ecc3c2dca1564a6dc))
- Add limits and tier to jwt claim ([#1382](https://github.com/shuttle-hq/shuttle/issues/1382)) - ([6a55b14](https://github.com/shuttle-hq/shuttle/commit/6a55b144f5ddd457ca5518bdef2ba6fa5097184c))

### Bug Fixes

- *(cargo-shuttle)* Revert shuttle-common-tests to path dep ([#1375](https://github.com/shuttle-hq/shuttle/issues/1375)) - ([70ba489](https://github.com/shuttle-hq/shuttle/commit/70ba489978aa506050317447e3704dffa4a011b3))
- *(gateway)* Use project_id argument when inserting project ([#1387](https://github.com/shuttle-hq/shuttle/issues/1387)) - ([96105b4](https://github.com/shuttle-hq/shuttle/commit/96105b4c8fd14642d4c70fcfdb231e1e5b8a0d65))

### Refactor

- Clean up deployer db resources, delete after sync to r-r, delete secrets command ([#1376](https://github.com/shuttle-hq/shuttle/issues/1376)) - ([03e7017](https://github.com/shuttle-hq/shuttle/commit/03e7017f7a6179b68dcecaf7355aa52686325c50))

### Documentation

- *(readme)* Add note about CCH to README.md ([#1389](https://github.com/shuttle-hq/shuttle/issues/1389)) - ([015717a](https://github.com/shuttle-hq/shuttle/commit/015717a2944064218ca2152c13e425280176ec99))

### Miscellaneous Tasks

- V0.33.0 ([#1390](https://github.com/shuttle-hq/shuttle/issues/1390)) - ([299a30d](https://github.com/shuttle-hq/shuttle/commit/299a30db5b3361c328b3231971d8276d10995be7))

### Miscellaneous

- *(changelog)* Add link to releases page ([#1378](https://github.com/shuttle-hq/shuttle/issues/1378)) - ([0526233](https://github.com/shuttle-hq/shuttle/commit/0526233bb6770a8d749266b52b119b9e326b706b))
- Bump axum minimum version, bump otel crates ([#1386](https://github.com/shuttle-hq/shuttle/issues/1386)) - ([3f4dc82](https://github.com/shuttle-hq/shuttle/commit/3f4dc822138b784ee23b42748830575be2ef0ec4))
- Rocket 0.5.0-rc.4 ([#1379](https://github.com/shuttle-hq/shuttle/issues/1379)) - ([22f512e](https://github.com/shuttle-hq/shuttle/commit/22f512e5553b9ab34d48b3f0222a00d94c4699f6))

## [0.32.0](https://github.com/shuttle-hq/shuttle/compare/v0.31.0..v0.32.0) - 2023-11-09

### Features

- *(installer)* Support installing the Alpine Linux package ([#1370](https://github.com/shuttle-hq/shuttle/issues/1370)) - ([d6e0c34](https://github.com/shuttle-hq/shuttle/commit/d6e0c345ea9f7822f9f80ff582adcb2af888a39f))
- Suggest project restart when trying to delete ([#1366](https://github.com/shuttle-hq/shuttle/issues/1366)) - ([3f14217](https://github.com/shuttle-hq/shuttle/commit/3f1421790639e10c0269730769779a468d1bf9c9))
- Use proto-gen for generating proto code ([#1364](https://github.com/shuttle-hq/shuttle/issues/1364)) - ([042c736](https://github.com/shuttle-hq/shuttle/commit/042c736af2f8782b00d0930160938008d89a9f6b))

### Bug Fixes

- Database uri password hiding, runtime version check ([#1368](https://github.com/shuttle-hq/shuttle/issues/1368)) - ([8bfdbc0](https://github.com/shuttle-hq/shuttle/commit/8bfdbc07b66e8faf99850ec4ea922e51603a7cb0))

### Refactor

- Make admin compile, scope project models to backends ([#1371](https://github.com/shuttle-hq/shuttle/issues/1371)) - ([0b35063](https://github.com/shuttle-hq/shuttle/commit/0b35063474e61e4e3fdce174b0106bd909f305ee))
- Fix ProjectName validation, custom Path extractor for parsing it ([#1354](https://github.com/shuttle-hq/shuttle/issues/1354)) - ([dd6b8fe](https://github.com/shuttle-hq/shuttle/commit/dd6b8feabad9950d0ef88148b5e314d9f7aa11f3))

### Documentation

- *(changelog)* Create CHANGELOG.md ([#1372](https://github.com/shuttle-hq/shuttle/issues/1372)) - ([019336e](https://github.com/shuttle-hq/shuttle/commit/019336e77ff47431b7e2013381ef0f0a85aa15c1))
- *(readme)* Add instructions for installing on Alpine Linux ([#1365](https://github.com/shuttle-hq/shuttle/issues/1365)) - ([a7b11a5](https://github.com/shuttle-hq/shuttle/commit/a7b11a54001c8c6c85d1e36cdf0fec1f08310b5b))

### Miscellaneous Tasks

- V0.32.0 ([#1373](https://github.com/shuttle-hq/shuttle/issues/1373)) - ([6943e21](https://github.com/shuttle-hq/shuttle/commit/6943e21341145e5335c140b93fb2b493ac6f2cf2))
- Audit on main, build release stack sooner, release crates faster ([#1369](https://github.com/shuttle-hq/shuttle/issues/1369)) - ([326e30a](https://github.com/shuttle-hq/shuttle/commit/326e30ab826e51bf143e9419b8283ca60d7405d5))
- Bump and refactor images, code cleanup ([#1313](https://github.com/shuttle-hq/shuttle/issues/1313)) - ([1c003cd](https://github.com/shuttle-hq/shuttle/commit/1c003cd2946c6fbc91f6923cb31d1a0e8e6087fd))

## [0.31.0](https://github.com/shuttle-hq/shuttle/compare/v0.30.1..v0.31.0) - 2023-11-02

### Features

- *(cargo-shuttle)* State MSRV in Cargo.toml ([#1356](https://github.com/shuttle-hq/shuttle/issues/1356)) - ([f4ddaa6](https://github.com/shuttle-hq/shuttle/commit/f4ddaa6c3527935953a82673eb5d3f0519eddd85))
- *(deployer)* Delete secrets from deployer persistence on secrets resource delete ([#1359](https://github.com/shuttle-hq/shuttle/issues/1359)) - ([e08dbec](https://github.com/shuttle-hq/shuttle/commit/e08dbeca6627fd1f5eeb469e6d4ea346ac6a0847))
- *(gateway)* Enforce project limits on project creation ([#1331](https://github.com/shuttle-hq/shuttle/issues/1331)) - ([574b7b8](https://github.com/shuttle-hq/shuttle/commit/574b7b86b78bf7c4f38e8804bf914238420cc8f4))
- *(logger)* Add instrumentation needed for alert ([#1348](https://github.com/shuttle-hq/shuttle/issues/1348)) - ([0d777cd](https://github.com/shuttle-hq/shuttle/commit/0d777cd9851f0f25f0b0bab4aee1b3e67ec4d178))
- Adapative page hints (client-side only) ([#1357](https://github.com/shuttle-hq/shuttle/issues/1357)) - ([ffb760a](https://github.com/shuttle-hq/shuttle/commit/ffb760abb92f644cde9c777b6973e412b46c8ebe))
- Wrap secrets in custom types to prevent them from leaking ([#925](https://github.com/shuttle-hq/shuttle/issues/925)) - ([bf6161c](https://github.com/shuttle-hq/shuttle/commit/bf6161cfeac9c11776b378bc8fbd156869fc5ad2))

### Bug Fixes

- *(deployer)* Handle cargo fetch without blocking logs, use async channels ([#1349](https://github.com/shuttle-hq/shuttle/issues/1349)) - ([03a8873](https://github.com/shuttle-hq/shuttle/commit/03a88730df797b7ca33f65f26e7e48f4d0f1f9c7))
- Typos found in codebase ([#1360](https://github.com/shuttle-hq/shuttle/issues/1360)) - ([78bd475](https://github.com/shuttle-hq/shuttle/commit/78bd475743ee4b2ff0f526cf5f291d7482193c71))

### Refactor

- *(runtime)* Hide internals from public-facing API, export tokio ([#1332](https://github.com/shuttle-hq/shuttle/issues/1332)) - ([06e46d1](https://github.com/shuttle-hq/shuttle/commit/06e46d158c84f4b1b5a38a8cb1ea00555c531a04))

### Documentation

- Add Docker Desktop config tip ([#1350](https://github.com/shuttle-hq/shuttle/issues/1350)) - ([fa29cee](https://github.com/shuttle-hq/shuttle/commit/fa29cee490805595cda0e75fc0b54a8cd7023ab3))
- Updates for return types for examples actix-web, tide, serenity, tower ([#892](https://github.com/shuttle-hq/shuttle/issues/892)) - ([3e63caa](https://github.com/shuttle-hq/shuttle/commit/3e63caa290c4e1e9fc745fb073ed9c5b39f098a3))

### Miscellaneous Tasks

- V0.31.0 ([#1361](https://github.com/shuttle-hq/shuttle/issues/1361)) - ([9366fc5](https://github.com/shuttle-hq/shuttle/commit/9366fc5365c58ccc6cdffcf11ad1708892bab6f9))
- Documentation updates for return types ([#893](https://github.com/shuttle-hq/shuttle/issues/893)) - ([b98ae53](https://github.com/shuttle-hq/shuttle/commit/b98ae53120e1c297a1d05fbd4d5eeaf6c6ea20e5))
- Add cargo-audit step ([#1345](https://github.com/shuttle-hq/shuttle/issues/1345)) - ([ccdb634](https://github.com/shuttle-hq/shuttle/commit/ccdb634929188d1d98c83952910ec92670379b7b))

### Miscellaneous

- Cargo update to fix audit issue ([#1358](https://github.com/shuttle-hq/shuttle/issues/1358)) - ([0bd7cd1](https://github.com/shuttle-hq/shuttle/commit/0bd7cd1fe6ebc7152286a6efe3e1a8ab3e148432))
- Revert turso version update ([#1355](https://github.com/shuttle-hq/shuttle/issues/1355)) - ([a0ae686](https://github.com/shuttle-hq/shuttle/commit/a0ae686f1d35ddf6790b426d29ba75abd30c20b5))
- Compile fails with secrets in resource configs ([#1353](https://github.com/shuttle-hq/shuttle/issues/1353)) - ([e79639e](https://github.com/shuttle-hq/shuttle/commit/e79639eb5bb634eabd3d113ce319aa6925bc047d))
- Resource provisioning errors not showing ([#1352](https://github.com/shuttle-hq/shuttle/issues/1352)) - ([04ded73](https://github.com/shuttle-hq/shuttle/commit/04ded73597165301900a2879efce3be31551b62d))
- Push renewed domain certificate to DB ([#1347](https://github.com/shuttle-hq/shuttle/issues/1347)) - ([74dbaa5](https://github.com/shuttle-hq/shuttle/commit/74dbaa534becc5c59b60df4a447916032fb7169c))

## [0.30.1](https://github.com/shuttle-hq/shuttle/compare/v0.30.0..v0.30.1) - 2023-10-24

### Bug Fixes

- Scope enum serialization compatibility ([#1341](https://github.com/shuttle-hq/shuttle/issues/1341)) - ([db63e66](https://github.com/shuttle-hq/shuttle/commit/db63e6680c49c58d899e6a2822d729b92cc29f8f))

### Miscellaneous Tasks

- V0.30.1 part 2 ([#1344](https://github.com/shuttle-hq/shuttle/issues/1344)) - ([dd55ca0](https://github.com/shuttle-hq/shuttle/commit/dd55ca08048acb6b6b8bf7d2f88958092b151212))
- V0.30.1 ([#1342](https://github.com/shuttle-hq/shuttle/issues/1342)) - ([7d04abb](https://github.com/shuttle-hq/shuttle/commit/7d04abb5b0c2ec01274d63febf366f0e9a5cfd2e))

## [0.30.0](https://github.com/shuttle-hq/shuttle/compare/v0.29.1..v0.30.0) - 2023-10-24

### Features

- *(cargo-shuttle)* Raw table output, fix table column alignment ([#1319](https://github.com/shuttle-hq/shuttle/issues/1319)) - ([cf7bcf7](https://github.com/shuttle-hq/shuttle/commit/cf7bcf70f9a683ca4898f8da196f758fd17e0888))
- *(logger)* Add basic instrumentation to the API ([#1336](https://github.com/shuttle-hq/shuttle/issues/1336)) - ([268f77f](https://github.com/shuttle-hq/shuttle/commit/268f77f35ceadd8a86ad6e13d49253ec44105cb7))
- *(resource-recorder)* Add basic instrumentation ([#1335](https://github.com/shuttle-hq/shuttle/issues/1335)) - ([f15fe92](https://github.com/shuttle-hq/shuttle/commit/f15fe92236b2c5d0f4a0d4db55649800b55eb563))
- Project delete ([#1307](https://github.com/shuttle-hq/shuttle/issues/1307)) - ([e9cf8fe](https://github.com/shuttle-hq/shuttle/commit/e9cf8febceddedd51595bf69e561243a8fabf53a))

### Bug Fixes

- *(installer)* Read input from process' controlling terminal ([#1327](https://github.com/shuttle-hq/shuttle/issues/1327)) - ([02369f5](https://github.com/shuttle-hq/shuttle/commit/02369f5122340ff0dd4b507bea3b03edd427c51d))

### Refactor

- *(deployer)* Improve deployment test failure msg ([#1326](https://github.com/shuttle-hq/shuttle/issues/1326)) - ([4306ae1](https://github.com/shuttle-hq/shuttle/commit/4306ae156697a2a1e48f97a4c0a73a5367ff9d02))

### Documentation

- *(cargo-shuttle)* Update the link for the Arch Linux package ([#1328](https://github.com/shuttle-hq/shuttle/issues/1328)) - ([42e9838](https://github.com/shuttle-hq/shuttle/commit/42e9838b87a76621d114703376f2ff73c5b58a43))

### Miscellaneous Tasks

- *(runtime)* Update to wasmtime 13.0 ([#1330](https://github.com/shuttle-hq/shuttle/issues/1330)) - ([b7c757c](https://github.com/shuttle-hq/shuttle/commit/b7c757ca4ae834d486ceff9267b253e9df182ed1))
- *(shuttle-turso)* Unpin libsql-client, bump to v0.32.0 ([#1329](https://github.com/shuttle-hq/shuttle/issues/1329)) - ([c84bd26](https://github.com/shuttle-hq/shuttle/commit/c84bd266dc22769a46b193cebad181cc2d9ab828))
- V0.30.0 ([#1339](https://github.com/shuttle-hq/shuttle/issues/1339)) - ([84ece38](https://github.com/shuttle-hq/shuttle/commit/84ece38a65df80b0e505d3ae30c5cbe357f95511))
- Update dependencies ([#1325](https://github.com/shuttle-hq/shuttle/issues/1325)) - ([186f1cd](https://github.com/shuttle-hq/shuttle/commit/186f1cd622238f85f8b6a48f1e3fd2cc94530fcd))
- Fix release ordering ([#1312](https://github.com/shuttle-hq/shuttle/issues/1312)) - ([793a3a4](https://github.com/shuttle-hq/shuttle/commit/793a3a46d94109c28e92e4055ad6dc157bf35ed0))

## [0.29.1](https://github.com/shuttle-hq/shuttle/compare/v0.29.0..v0.29.1) - 2023-10-12

### Bug Fixes

- *(cargo-shuttle)* Wait for ready again after db reboot ([#1314](https://github.com/shuttle-hq/shuttle/issues/1314)) - ([33b13b5](https://github.com/shuttle-hq/shuttle/commit/33b13b510e07ebf5207a4a311e9cfa41936dd3a5))

### Miscellaneous Tasks

- V0.29.1 ([#1317](https://github.com/shuttle-hq/shuttle/issues/1317)) - ([7ef6395](https://github.com/shuttle-hq/shuttle/commit/7ef6395e1bed53c7bd57925cd189db7e8d9843c1))

## [0.29.0](https://github.com/shuttle-hq/shuttle/compare/v0.28.1..v0.29.0) - 2023-10-09

### Features

- *(auth)* Added billing backend support ([#1289](https://github.com/shuttle-hq/shuttle/issues/1289)) - ([b37b03f](https://github.com/shuttle-hq/shuttle/commit/b37b03ff8cd979f69538792fde698cec5acea0cf))
- Implement resource deletion ([#1256](https://github.com/shuttle-hq/shuttle/issues/1256)) - ([c65a897](https://github.com/shuttle-hq/shuttle/commit/c65a89789345d58b89571b390d860767f668ea02))

### Bug Fixes

- *(cargo-shuttle)* Fix init login bugs ([#1309](https://github.com/shuttle-hq/shuttle/issues/1309)) - ([41ed35d](https://github.com/shuttle-hq/shuttle/commit/41ed35d5309276b4959f16b686b5258a88d0f1d1))
- *(cargo-shuttle)* Logout command needs client. - ([ee47b11](https://github.com/shuttle-hq/shuttle/commit/ee47b115430fbd73c91ebe85ab706aa487740624))

### Miscellaneous Tasks

- *(editorconfig)* Add indentation settings for shell scripts ([#1296](https://github.com/shuttle-hq/shuttle/issues/1296)) - ([72f8484](https://github.com/shuttle-hq/shuttle/commit/72f8484074a1fe66ee5bf9f66804402cb3bda19c))
- V0.29.0 ([#1310](https://github.com/shuttle-hq/shuttle/issues/1310)) - ([8749a06](https://github.com/shuttle-hq/shuttle/commit/8749a06e72d7cb07e5c6971c9ccbeb4aca2de4f2))
- Remove shuttle-static-folder ([#1298](https://github.com/shuttle-hq/shuttle/issues/1298)) - ([8e466f0](https://github.com/shuttle-hq/shuttle/commit/8e466f0f93620a6d1d4468ca40390497b1152ad8))
- Use smaller machines for docker tests ([#1301](https://github.com/shuttle-hq/shuttle/issues/1301)) - ([629a63c](https://github.com/shuttle-hq/shuttle/commit/629a63c024956a62b802573a800dafdeee350f56))
- Fix cargo-shuttle publish ordering ([#1297](https://github.com/shuttle-hq/shuttle/issues/1297)) - ([6ffc717](https://github.com/shuttle-hq/shuttle/commit/6ffc7175d014b6e36456e090d458e1b9a11b0142))

## [0.28.1](https://github.com/shuttle-hq/shuttle/compare/v0.28.0..v0.28.1) - 2023-10-05

### Bug Fixes

- Cargo-shuttle panic on the login command ([#1302](https://github.com/shuttle-hq/shuttle/issues/1302)) - ([bdbf92f](https://github.com/shuttle-hq/shuttle/commit/bdbf92f00700d8a507fa6f75a37a4cc89c8bfffc))

### Testing

- *(cargo-shuttle)* Add debug assertion for command-line arguments ([#1295](https://github.com/shuttle-hq/shuttle/issues/1295)) - ([faaf9e8](https://github.com/shuttle-hq/shuttle/commit/faaf9e88ee66475eacaaba033bcbcd5ea926f195))

## [0.28.0](https://github.com/shuttle-hq/shuttle/compare/v0.27.0..v0.28.0) - 2023-10-03

### Features

- *(builder)* Improve the nix build capturing of stdout/stderr ([#1268](https://github.com/shuttle-hq/shuttle/issues/1268)) - ([0b577b9](https://github.com/shuttle-hq/shuttle/commit/0b577b9847442d31c03ae83619ee63829b717c25))
- *(cargo-shuttle)* Check project name available ([#1279](https://github.com/shuttle-hq/shuttle/issues/1279)) - ([da18b3b](https://github.com/shuttle-hq/shuttle/commit/da18b3b927148c2075e33fcd5ce1b01039cbddf0))
- *(cargo-shuttle)* Better compression & handling of config files after init ([#1257](https://github.com/shuttle-hq/shuttle/issues/1257)) - ([ce5f234](https://github.com/shuttle-hq/shuttle/commit/ce5f234c7044fe58e5618c5e5e92e15001625a76))
- *(ci)* Separation of tests that need docker ([#1249](https://github.com/shuttle-hq/shuttle/issues/1249)) - ([a0ad7a2](https://github.com/shuttle-hq/shuttle/commit/a0ad7a2f927d6d9252144ed116f375730e0eb3f7))
- *(installer)* Add installer script ([#1280](https://github.com/shuttle-hq/shuttle/issues/1280)) - ([0935c27](https://github.com/shuttle-hq/shuttle/commit/0935c27a2348fdd11d5dbe7c8adba0366ffa508c))
- *(orchestrator)* Initialize shuttle-orchestrator as a library ([#1271](https://github.com/shuttle-hq/shuttle/issues/1271)) - ([6b8b62c](https://github.com/shuttle-hq/shuttle/commit/6b8b62ce45d1c8f990df3632fc14c65740f30a17))
- Add lld and mold linkers ([#1286](https://github.com/shuttle-hq/shuttle/issues/1286)) - ([3caeb8b](https://github.com/shuttle-hq/shuttle/commit/3caeb8b609090181f0273a0c7510b7a7a03b1183))
- Use smaller+newer images, script for patches, unique binary names ([#1247](https://github.com/shuttle-hq/shuttle/issues/1247)) - ([9c01cbe](https://github.com/shuttle-hq/shuttle/commit/9c01cbe334fc166c7439ce6d4591c88fc544c03c))
- Version checks between cli, gateway, deployer, runtime ([#1275](https://github.com/shuttle-hq/shuttle/issues/1275)) - ([538473a](https://github.com/shuttle-hq/shuttle/commit/538473a58661be0ba243fa4b551b376105c21ac8))

### Bug Fixes

- *(cargo-shuttle)* Spam less requests when waiting for project ready ([#1287](https://github.com/shuttle-hq/shuttle/issues/1287)) - ([20def88](https://github.com/shuttle-hq/shuttle/commit/20def881f80379f46548744c6fef64801b92fe07))
- *(cargo-shuttle)* Prompt for new port if port is taken ([#1270](https://github.com/shuttle-hq/shuttle/issues/1270)) - ([ff6fe3b](https://github.com/shuttle-hq/shuttle/commit/ff6fe3baf189e359d9fb7fbcc09c96728f64f46b))
- *(deployer)* Added runtime error handling ([#1231](https://github.com/shuttle-hq/shuttle/issues/1231)) - ([b0d79a4](https://github.com/shuttle-hq/shuttle/commit/b0d79a42e124a8d696692469f54f49443facff96))
- *(deployer)* Handle gracefully builder connection failure ([#1264](https://github.com/shuttle-hq/shuttle/issues/1264)) - ([7991c58](https://github.com/shuttle-hq/shuttle/commit/7991c58cbbaa326f524c0b6596cf255b83915d66))
- *(docker-compose.dev)* Adjust auth dev dependency ([#1274](https://github.com/shuttle-hq/shuttle/issues/1274)) - ([e78f6c7](https://github.com/shuttle-hq/shuttle/commit/e78f6c7f2859ba897d66d7bb40c832e5ff5d3e60))
- *(gateway)* Install curl for health checks ([#1291](https://github.com/shuttle-hq/shuttle/issues/1291)) - ([b623b28](https://github.com/shuttle-hq/shuttle/commit/b623b2855d4429e4a7b5a9b3f2acf5f42928c1ee))
- Cleanup for 0.28.0 ([#1278](https://github.com/shuttle-hq/shuttle/issues/1278)) - ([88e7519](https://github.com/shuttle-hq/shuttle/commit/88e751990c7221a3dc34042c519bad2d56b055ac))
- Fix compose starup - ([03acbae](https://github.com/shuttle-hq/shuttle/commit/03acbaed34505c953f9dff5fa1addc8822d77be9))
- Gateway container startup on apple m1 ([#1284](https://github.com/shuttle-hq/shuttle/issues/1284)) - ([680f7a7](https://github.com/shuttle-hq/shuttle/commit/680f7a77eb9b2550b19d0b5cf614c602ac94cbba))

### Refactor

- *(cargo-shuttle)* Remove `cargo-generate` dependency ([#1281](https://github.com/shuttle-hq/shuttle/issues/1281)) - ([b18b7f3](https://github.com/shuttle-hq/shuttle/commit/b18b7f3a97fa1f176af9382afc0933e6825116a7))

### Documentation

- Add installer script option ([#1290](https://github.com/shuttle-hq/shuttle/issues/1290)) - ([291f5f6](https://github.com/shuttle-hq/shuttle/commit/291f5f62c61da32d94074e749909593bdf64cac0))

### Miscellaneous Tasks

- *(docker)* Set up a local shared postgres for development ([#1272](https://github.com/shuttle-hq/shuttle/issues/1272)) - ([7159f9c](https://github.com/shuttle-hq/shuttle/commit/7159f9c6205abac6d164b95594fc9d65ff59e9f5))
- V0.28.0 ([#1293](https://github.com/shuttle-hq/shuttle/issues/1293)) - ([4707027](https://github.com/shuttle-hq/shuttle/commit/4707027d0fcda8d77635f97d606db346e0b6ae1b))
- Separate ci and unstable jobs, better caching ([#1273](https://github.com/shuttle-hq/shuttle/issues/1273)) - ([854df3f](https://github.com/shuttle-hq/shuttle/commit/854df3f49959834f4df0382f55f327cafaadc380))
- Reduce shortest path in publish flow ([#1265](https://github.com/shuttle-hq/shuttle/issues/1265)) - ([bd36b9c](https://github.com/shuttle-hq/shuttle/commit/bd36b9cca205ce95f1cb64c6bd2e545c7eda1562))

### Miscellaneous

- Projects' states drifting ([#1262](https://github.com/shuttle-hq/shuttle/issues/1262)) - ([974b99c](https://github.com/shuttle-hq/shuttle/commit/974b99c2b181c57697ca55b03a2652e8e6b21e0a))

## [0.27.0](https://github.com/shuttle-hq/shuttle/compare/v0.26.0..v0.27.0) - 2023-09-21

### Features

- *(builder)* Update tracing logs ([#1252](https://github.com/shuttle-hq/shuttle/issues/1252)) - ([de2603e](https://github.com/shuttle-hq/shuttle/commit/de2603e93e230bb711ee4c053e63177f58ea9726))
- *(cargo-shuttle)* Add suggestions in case of cmd failures ([#1245](https://github.com/shuttle-hq/shuttle/issues/1245)) - ([27092b8](https://github.com/shuttle-hq/shuttle/commit/27092b8bec8e2b98c4629d26046356df7e305f0d))
- *(deployer)* Send deployment archive to the builder ([#1253](https://github.com/shuttle-hq/shuttle/issues/1253)) - ([a273224](https://github.com/shuttle-hq/shuttle/commit/a27322446ff6fd3641cae1c65ea7bd1f12906c42))
- *(deployer)* Connect deployer to builder service ([#1248](https://github.com/shuttle-hq/shuttle/issues/1248)) - ([97077ae](https://github.com/shuttle-hq/shuttle/commit/97077ae67727e90b8ebc6144d457a1445d6fd961))
- Execute projects from within workspace and other resources changes ([#1050](https://github.com/shuttle-hq/shuttle/issues/1050)) - ([9d28100](https://github.com/shuttle-hq/shuttle/commit/9d28100081ff71e906b8f8bbbb4ac6e8ec905a3e))
- Builder service ([#1244](https://github.com/shuttle-hq/shuttle/issues/1244)) - ([361e00e](https://github.com/shuttle-hq/shuttle/commit/361e00ec41c2bcce3a2fc87d910d4524cf60646c))

### Bug Fixes

- *(cargo-shuttle)* Add helpful error message on docker container error ([#951](https://github.com/shuttle-hq/shuttle/issues/951)) - ([7b31aba](https://github.com/shuttle-hq/shuttle/commit/7b31abafcad40cd7e059fb537cfdb7a8dae987d0))
- *(cargo-shuttle)* Secrets project requires a Secrets.toml ([#1250](https://github.com/shuttle-hq/shuttle/issues/1250)) - ([0283c3a](https://github.com/shuttle-hq/shuttle/commit/0283c3a4647cffd68de80aa04cd7618bc235a264))
- *(shuttle-metadata)* Metadata re-export ([#1255](https://github.com/shuttle-hq/shuttle/issues/1255)) - ([c7eb3b5](https://github.com/shuttle-hq/shuttle/commit/c7eb3b53989ce7c3137e384d75001c7a5e470a3e))
- Default network subnet overlap ([#1254](https://github.com/shuttle-hq/shuttle/issues/1254)) - ([0bbadff](https://github.com/shuttle-hq/shuttle/commit/0bbadffd61ce7fd65d19fdaf73894ffa1737f2c5))

### Miscellaneous Tasks

- *(shuttle-shared-db)* Bump local postgres version from 11 to 14 ([#1073](https://github.com/shuttle-hq/shuttle/issues/1073)) - ([0d64923](https://github.com/shuttle-hq/shuttle/commit/0d64923f7598ce65e603589b0cedbd08fc8a6101))
- V0.27.0 ([#1261](https://github.com/shuttle-hq/shuttle/issues/1261)) - ([2af0076](https://github.com/shuttle-hq/shuttle/commit/2af0076623fa5bb5fd98524642c5afc8b98eab66))

### Miscellaneous

- Project entering a state loop ([#1260](https://github.com/shuttle-hq/shuttle/issues/1260)) - ([6b73157](https://github.com/shuttle-hq/shuttle/commit/6b731577593a4de5ad0a6c90d32ae0efedb6eafd))
- Fix Cargo.lock - ([9889b59](https://github.com/shuttle-hq/shuttle/commit/9889b59bdbf73df20f6cc552d4cff591e6c791f1))

## [0.26.0](https://github.com/shuttle-hq/shuttle/compare/v0.25.1..v0.26.0) - 2023-09-18

### Features

- *(cargo-shuttle)* Prompt for init path when not given, warn if init dir not empty ([#1198](https://github.com/shuttle-hq/shuttle/issues/1198)) - ([e1d263e](https://github.com/shuttle-hq/shuttle/commit/e1d263ec44e1d5a2ebafc1ec35ce09a52d1c1c97))
- *(common)* Change request_span to info ([#1230](https://github.com/shuttle-hq/shuttle/issues/1230)) - ([da71952](https://github.com/shuttle-hq/shuttle/commit/da7195258edb98e6d483ead8971e381d908860d7))
- *(containerfile)* Improve deployer build caching ([#1214](https://github.com/shuttle-hq/shuttle/issues/1214)) - ([9f3aeb9](https://github.com/shuttle-hq/shuttle/commit/9f3aeb99c44f0e510b660fac7eb99fc647ac4a08))
- *(gateway)* Inform project owner about running state ([#1194](https://github.com/shuttle-hq/shuttle/issues/1194)) - ([2fa1db3](https://github.com/shuttle-hq/shuttle/commit/2fa1db3ef77f739db4f870dc9e2eaab0f2d9f4be))
- *(gateway)* Special error if own project is already running ([#1192](https://github.com/shuttle-hq/shuttle/issues/1192)) - ([5a66ca5](https://github.com/shuttle-hq/shuttle/commit/5a66ca5ab9e2f146661c3ccbd72d9ede61a24a73))
- *(logger)* Logger broadcast channel queue size traces ([#1235](https://github.com/shuttle-hq/shuttle/issues/1235)) - ([c2c4ca0](https://github.com/shuttle-hq/shuttle/commit/c2c4ca0ce6ee726c32675a384784d6d97774a225))
- *(logger)* Refactor to loop, add traces ([#1232](https://github.com/shuttle-hq/shuttle/issues/1232)) - ([57f5b15](https://github.com/shuttle-hq/shuttle/commit/57f5b1539cbc7a25fdabad60bdbe9c2fb4497cb7))
- *(services)* Enable auto-sharding in shuttle-poise ([#1217](https://github.com/shuttle-hq/shuttle/issues/1217)) - ([32d63eb](https://github.com/shuttle-hq/shuttle/commit/32d63ebdc44ffe23e6077d3609733777b6e6d554))
- *(shuttle-next)* Enable tracing by default ([#1219](https://github.com/shuttle-hq/shuttle/issues/1219)) - ([ef47eae](https://github.com/shuttle-hq/shuttle/commit/ef47eaeea61836d8c3a60d765509084829872b4e))
- Outdated log parse warning ([#1243](https://github.com/shuttle-hq/shuttle/issues/1243)) - ([a77ecb1](https://github.com/shuttle-hq/shuttle/commit/a77ecb1732e8ae8813428050583216ac0ea65db0))
- Match local logs with deployer logs ([#1216](https://github.com/shuttle-hq/shuttle/issues/1216)) - ([1d13115](https://github.com/shuttle-hq/shuttle/commit/1d131150976a77973a81e480f1bcb936ede07759))

### Bug Fixes

- *(Containerfile)* Copied shuttle-logger service in the final image ([#1242](https://github.com/shuttle-hq/shuttle/issues/1242)) - ([c7ac99b](https://github.com/shuttle-hq/shuttle/commit/c7ac99b541ec353f288446c51d2503b3e541a859))
- *(deployer)* Handle errors from corrupted resource data ([#1208](https://github.com/shuttle-hq/shuttle/issues/1208)) - ([d7b5b6a](https://github.com/shuttle-hq/shuttle/commit/d7b5b6afdcdcfcf2a5a3cd9dad5b1728f16b0036))
- *(logger)* Resolve CI failures caused by recent changes ([#1212](https://github.com/shuttle-hq/shuttle/issues/1212)) - ([8c8e338](https://github.com/shuttle-hq/shuttle/commit/8c8e338715a4b59d0305562ba20d8637b0ead0f4))
- *(otel)* Restore honeycomb and dd exporters ([#1218](https://github.com/shuttle-hq/shuttle/issues/1218)) - ([9249beb](https://github.com/shuttle-hq/shuttle/commit/9249beba3cf0c1523a9990f9e9cf5cf1ca9e8573))
- *(persist)* Don't use lifetime in error ([#1195](https://github.com/shuttle-hq/shuttle/issues/1195)) - ([6ff0f19](https://github.com/shuttle-hq/shuttle/commit/6ff0f19997586429c9777b51ef16be98ba08928d))
- Remove duplicate makefile command, move .so copy in containerfile ([#1241](https://github.com/shuttle-hq/shuttle/issues/1241)) - ([634679a](https://github.com/shuttle-hq/shuttle/commit/634679aab06a5c5e65ceb33bf7a114fa89131860))
- Truncate log item strings ([#1227](https://github.com/shuttle-hq/shuttle/issues/1227)) - ([30c0bde](https://github.com/shuttle-hq/shuttle/commit/30c0bdeadd3e12c8e1552dd4fafb8474b31e044e))
- Logger branch cleanups ([#1226](https://github.com/shuttle-hq/shuttle/issues/1226)) - ([395624b](https://github.com/shuttle-hq/shuttle/commit/395624b0b0925b6225d35d8ca778eab76a335f45))
- Span names, log levels and messages ([#1213](https://github.com/shuttle-hq/shuttle/issues/1213)) - ([b8bedf7](https://github.com/shuttle-hq/shuttle/commit/b8bedf78a648df67e3047b3d4dd3653447093aa1))
- Missing readmes in deployers local source ([#1206](https://github.com/shuttle-hq/shuttle/issues/1206)) - ([ce4a6ec](https://github.com/shuttle-hq/shuttle/commit/ce4a6ec185d4b60b3f4601f6f7fb4224efef0c00))

### Refactor

- Add index to deployment id ([#1224](https://github.com/shuttle-hq/shuttle/issues/1224)) - ([c2a4892](https://github.com/shuttle-hq/shuttle/commit/c2a4892a33317ab4f3f4167af1c3654b0dc9011d))
- Improve stream logs ([#1221](https://github.com/shuttle-hq/shuttle/issues/1221)) - ([91e9239](https://github.com/shuttle-hq/shuttle/commit/91e9239f05f937f6c8f736a2012818f930197d21))
- Switch to LOGGER_POSTGRES_URI ([#1220](https://github.com/shuttle-hq/shuttle/issues/1220)) - ([d94a7ee](https://github.com/shuttle-hq/shuttle/commit/d94a7ee7a14831b89785479dd579454c6d5e456c))

### Miscellaneous Tasks

- *(changelog)* Add git-cliff configuration ([#1200](https://github.com/shuttle-hq/shuttle/issues/1200)) - ([b3de162](https://github.com/shuttle-hq/shuttle/commit/b3de162b8a9c9597f46b7a38618b2930068a6aa0))
- *(makefile)* Remove unused commands ([#1196](https://github.com/shuttle-hq/shuttle/issues/1196)) - ([a9ffc8f](https://github.com/shuttle-hq/shuttle/commit/a9ffc8f7be326d62a41831c1af8b159a645d0551))
- Bump examples ([#1246](https://github.com/shuttle-hq/shuttle/issues/1246)) - ([c7c0ceb](https://github.com/shuttle-hq/shuttle/commit/c7c0ceb59be91b986169bf624f2e99ed806c7345))
- V0.26.0 ([#1239](https://github.com/shuttle-hq/shuttle/issues/1239)) - ([94f7966](https://github.com/shuttle-hq/shuttle/commit/94f79662bd0f99b9cc229a62ce78b5ad725f38e8))
- Uncomment build & deploy branch filters ([#1238](https://github.com/shuttle-hq/shuttle/issues/1238)) - ([7703d85](https://github.com/shuttle-hq/shuttle/commit/7703d85685462b08ba2d0fb7927fa98cb0501455))
- Logger postgres uri ([#1228](https://github.com/shuttle-hq/shuttle/issues/1228)) - ([4fb7629](https://github.com/shuttle-hq/shuttle/commit/4fb762961eb63d817e7312f87b12fe6060f8f867))
- Update readme with new persist methods ([#1184](https://github.com/shuttle-hq/shuttle/issues/1184)) - ([d30c9a4](https://github.com/shuttle-hq/shuttle/commit/d30c9a4649d0c6c8b7cbb9a6e9b79df1cf4a5024))

### Miscellaneous

- Merge pull request #1225 from shuttle-hq/feat/shuttle-logger-service - ([a74153a](https://github.com/shuttle-hq/shuttle/commit/a74153a81378949300255b1ff005ca911f77b49a))
- Merge branch 'main' into feat/shuttle-logger-service - ([2421117](https://github.com/shuttle-hq/shuttle/commit/2421117e0e5f40244bcc3eab8d7feb4aff1ec761))
- Batch in 1 sec intervals ([#1222](https://github.com/shuttle-hq/shuttle/issues/1222)) - ([2c5a0bb](https://github.com/shuttle-hq/shuttle/commit/2c5a0bb301831e4a2f584098cf20641e880502dd))
- Merge remote-tracking branch 'upstream/main' into feat/shuttle-logger-service - ([4b4b7b3](https://github.com/shuttle-hq/shuttle/commit/4b4b7b38ba8f00a31e753bd18e6c2b7b1269be60))
- Revert "feat(shuttle-axum) Make AxumService generic to be able to use axum::State with it ([#924](https://github.com/shuttle-hq/shuttle/issues/924))" ([#1199](https://github.com/shuttle-hq/shuttle/issues/1199)) - ([fa86d5b](https://github.com/shuttle-hq/shuttle/commit/fa86d5b1830d701da6f75a0ab7075d642332d112))

## [0.25.1](https://github.com/shuttle-hq/shuttle/compare/v0.25.0..v0.25.1) - 2023-08-28

### Bug Fixes

- Cargo-shuttle missing feature ([#1178](https://github.com/shuttle-hq/shuttle/issues/1178)) - ([edf687f](https://github.com/shuttle-hq/shuttle/commit/edf687fdc91a3cb4792901c13cab089ad2db1a0c))

### Miscellaneous Tasks

- Cargo-shuttle v0.25.1 ([#1182](https://github.com/shuttle-hq/shuttle/issues/1182)) - ([9dd4bbf](https://github.com/shuttle-hq/shuttle/commit/9dd4bbfd1c05c9089c2cf59dbae005d3972d3371))
- Fix invalid resource-class for deploy job ([#1180](https://github.com/shuttle-hq/shuttle/issues/1180)) - ([b774a54](https://github.com/shuttle-hq/shuttle/commit/b774a54d0f8e23133f5837d42b7b80ba671f0d32))

## [0.25.0](https://github.com/shuttle-hq/shuttle/compare/v0.24.0..v0.25.0) - 2023-08-28

### Features

- *(codegen)* Restore default log level, improve error messages ([#1211](https://github.com/shuttle-hq/shuttle/issues/1211)) - ([e8e0f12](https://github.com/shuttle-hq/shuttle/commit/e8e0f12fc8717259cc49a82d3789d31e4988f6e1))
- *(deployer)* StateChangeLayer, DeploymentLogLayer, new log item structure ([#1171](https://github.com/shuttle-hq/shuttle/issues/1171)) - ([7ab8d11](https://github.com/shuttle-hq/shuttle/commit/7ab8d111f385454381f971705ce7f3d759eb03d7))
- *(deployer)* Send runtime logs to the logger service ([#1173](https://github.com/shuttle-hq/shuttle/issues/1173)) - ([4541ef6](https://github.com/shuttle-hq/shuttle/commit/4541ef6de4f79781ddf2037bf2558f01a6da2a27))
- *(runtime)* Set up a tracing-subscriber as a default feature ([#1203](https://github.com/shuttle-hq/shuttle/issues/1203)) - ([3d2feca](https://github.com/shuttle-hq/shuttle/commit/3d2feca7a878eed8deb4c7407a050c2d7bb2277c))
- *(runtime)* Write next runtime logs to stdout ([#1187](https://github.com/shuttle-hq/shuttle/issues/1187)) - ([0f269d6](https://github.com/shuttle-hq/shuttle/commit/0f269d67ab2c1c27575f317c0f5bbb464582adb9))
- Logs batching ([#1188](https://github.com/shuttle-hq/shuttle/issues/1188)) - ([64520fb](https://github.com/shuttle-hq/shuttle/commit/64520fb14d87a2c5dfc3194ce857a989181d90b3))
- Add idle timeout warning on project creation ([#1116](https://github.com/shuttle-hq/shuttle/issues/1116)) - ([28d1a7a](https://github.com/shuttle-hq/shuttle/commit/28d1a7a7ce4a5c384aac24fef1112afe2b7995ad))
- Add service-info resource to obtain Shuttle service info ([#1129](https://github.com/shuttle-hq/shuttle/issues/1129)) - ([dbb9adb](https://github.com/shuttle-hq/shuttle/commit/dbb9adb9f0b5e3cf4a88bfc599d0b59e3f719b52))
- Merge logger service from feat/shuttle-runtime-scaling ([#1139](https://github.com/shuttle-hq/shuttle/issues/1139)) - ([d8945d8](https://github.com/shuttle-hq/shuttle/commit/d8945d84cf22fd8a5ec23e5fd770181158b1ba5f))

### Bug Fixes

- Unused sqlx dep ([#1157](https://github.com/shuttle-hq/shuttle/issues/1157)) - ([531014e](https://github.com/shuttle-hq/shuttle/commit/531014eaf4b38e1b3844324bb7154c6128228bb8))

### Refactor

- *(proto)* Fix the use of deprecated chrono datetime ([#1207](https://github.com/shuttle-hq/shuttle/issues/1207)) - ([9f391ee](https://github.com/shuttle-hq/shuttle/commit/9f391eeb5a03c1489e3d497a981ee0d2dc410e65))
- *(runtime)* Replace trace with println ([#1190](https://github.com/shuttle-hq/shuttle/issues/1190)) - ([786c2dd](https://github.com/shuttle-hq/shuttle/commit/786c2dd85c937690a4f1bad8dd6e57bce2a87a4b))
- *(runtime,codegen)* Avoid double timestamps problem ([#1210](https://github.com/shuttle-hq/shuttle/issues/1210)) - ([a7d0ee0](https://github.com/shuttle-hq/shuttle/commit/a7d0ee000350e3ffbd988883b55675e89df57258))
- Remove println from logger ([#1186](https://github.com/shuttle-hq/shuttle/issues/1186)) - ([8bd9ff2](https://github.com/shuttle-hq/shuttle/commit/8bd9ff2fb7e9ba71231cef0933919325d6e1c0b7))
- Remove tracing from runtime ([#1185](https://github.com/shuttle-hq/shuttle/issues/1185)) - ([5fda73c](https://github.com/shuttle-hq/shuttle/commit/5fda73c121b83f0a682de154697ab1de4d3be8c7))
- Containerfile+Makefile improvement: build crates together, then distribute the binaries ([#1164](https://github.com/shuttle-hq/shuttle/issues/1164)) - ([6ccf54c](https://github.com/shuttle-hq/shuttle/commit/6ccf54c0fef8a597488090970c0473f642010a7f))
- Reduce noise in honeycomb ([#1142](https://github.com/shuttle-hq/shuttle/issues/1142)) - ([c1d05d8](https://github.com/shuttle-hq/shuttle/commit/c1d05d82b79b886febba8f6355c62fb3263b8387))

### Testing

- *(deployer)* Fixed deployer tests and removed unnecessary runtime logger_uri arg ([#1204](https://github.com/shuttle-hq/shuttle/issues/1204)) - ([86767f8](https://github.com/shuttle-hq/shuttle/commit/86767f8828859217cc7530e75d9546ce0c05a0ed))

### Miscellaneous Tasks

- *(gateway)* Stop setting `RUST_LOG` in deployers ([#1197](https://github.com/shuttle-hq/shuttle/issues/1197)) - ([bd5c9ff](https://github.com/shuttle-hq/shuttle/commit/bd5c9ff5a1159b374515d624e4ced25c5cc2eda5))
- *(resources)* Rename service-info to metadata ([#1165](https://github.com/shuttle-hq/shuttle/issues/1165)) - ([07e2566](https://github.com/shuttle-hq/shuttle/commit/07e256677621cfdefecf4cdb0211a4e50248ef2b))
- *(services)* Disable default features for shuttle-runtime ([#1205](https://github.com/shuttle-hq/shuttle/issues/1205)) - ([b158bca](https://github.com/shuttle-hq/shuttle/commit/b158bca50c9cc57c3d77eef09ec6254e2c4adb90))
- V0.25.0 ([#1175](https://github.com/shuttle-hq/shuttle/issues/1175)) - ([dbb468d](https://github.com/shuttle-hq/shuttle/commit/dbb468dc42fbadc32bd0d74f5a01c807c313614e))
- Rust 1.72.0 ([#1176](https://github.com/shuttle-hq/shuttle/issues/1176)) - ([4b32d38](https://github.com/shuttle-hq/shuttle/commit/4b32d38238f83c7cb037ee2275f63b9b566761c3))
- Simplify contributor list ([#1170](https://github.com/shuttle-hq/shuttle/issues/1170)) - ([535de7a](https://github.com/shuttle-hq/shuttle/commit/535de7afe013962f56519a6ff2a44226da9a767d))
- Create the local setup for replacing shuttle-logger sqlite with postgres ([#1145](https://github.com/shuttle-hq/shuttle/issues/1145)) - ([ffd1b13](https://github.com/shuttle-hq/shuttle/commit/ffd1b131abf73c351e297bf04dd52ca1f294bcff))
- Adjust logger to receive logs blobs ([#1172](https://github.com/shuttle-hq/shuttle/issues/1172)) - ([149e9b0](https://github.com/shuttle-hq/shuttle/commit/149e9b039be7de1ae826d97b99a21f49ef48b910))
- Refactor and improve speed and caching. add rustls flags to aws-rds ([#1167](https://github.com/shuttle-hq/shuttle/issues/1167)) - ([de362f8](https://github.com/shuttle-hq/shuttle/commit/de362f8269e3a879c635f1e9c71400baae1bc441))
- Improve development docs & scripts ([#1156](https://github.com/shuttle-hq/shuttle/issues/1156)) - ([b7fb596](https://github.com/shuttle-hq/shuttle/commit/b7fb596782c6b96805c96e8f049ef55714780e04))
- Update labels ([#1161](https://github.com/shuttle-hq/shuttle/issues/1161)) - ([eb93d0f](https://github.com/shuttle-hq/shuttle/commit/eb93d0fd0621e334b622080c2bba31475b258765))
- Add list, remove, clear, and size operations to shuttle-persist ([#1066](https://github.com/shuttle-hq/shuttle/issues/1066)) - ([31dec11](https://github.com/shuttle-hq/shuttle/commit/31dec115b16b604b3ecc52be11991434b40a18b7))

### Miscellaneous

- Service name being unknown ([#1202](https://github.com/shuttle-hq/shuttle/issues/1202)) - ([d661143](https://github.com/shuttle-hq/shuttle/commit/d6611434252094eff71482d55698d03e79f94263))
- Merge remote-tracking branch 'upstream/feat/shuttle-logger-service' into feat/shuttle-logger-service - ([bb62a4f](https://github.com/shuttle-hq/shuttle/commit/bb62a4f1e6a86e42d4721e534499639b9c9f5bf2))
- Merge remote-tracking branch 'upstream/main' into feat/shuttle-logger-service - ([2314c12](https://github.com/shuttle-hq/shuttle/commit/2314c1221952064c0e4581cc7c5f81015acc370b))
- Add suggestion to 'project not ready' error message ([#1169](https://github.com/shuttle-hq/shuttle/issues/1169)) - ([2c86a52](https://github.com/shuttle-hq/shuttle/commit/2c86a5275ed12bd3fb54519f34e1ee5b8ef2ddfa))
- Fix custom domains request/renew APIs ([#1158](https://github.com/shuttle-hq/shuttle/issues/1158)) - ([38f42bd](https://github.com/shuttle-hq/shuttle/commit/38f42bd81543421659bbbe0044c79b561a03f116))
- Store span names  ([#1166](https://github.com/shuttle-hq/shuttle/issues/1166)) - ([f04245f](https://github.com/shuttle-hq/shuttle/commit/f04245f5047615049b7943661a8dc169a6be3f3a))
- Update logs APIs  to fetch the logs from shuttle-logger ([#1143](https://github.com/shuttle-hq/shuttle/issues/1143)) - ([4c83051](https://github.com/shuttle-hq/shuttle/commit/4c830514a65bbd89f7b8335f063ee02725146eac))

## [0.24.0](https://github.com/shuttle-hq/shuttle/compare/v0.23.0..v0.24.0) - 2023-08-16

### Features

- Custom tracing layer ([#1027](https://github.com/shuttle-hq/shuttle/issues/1027)) - ([fe2f47d](https://github.com/shuttle-hq/shuttle/commit/fe2f47dc6cf358114dfbd457cb2adb84854f1c8e))
- Redirect user to console page on CLI login ([#1069](https://github.com/shuttle-hq/shuttle/issues/1069)) - ([e9aeaaf](https://github.com/shuttle-hq/shuttle/commit/e9aeaafbd6cea8f07fb056aff90eeab76f0d923d))

### Bug Fixes

- Static folder is not updated ([#1151](https://github.com/shuttle-hq/shuttle/issues/1151)) - ([a64e7af](https://github.com/shuttle-hq/shuttle/commit/a64e7af1f5b745c536ca71ba13c077aac9b77413))
- Secrets not updating ([#1150](https://github.com/shuttle-hq/shuttle/issues/1150)) - ([451387b](https://github.com/shuttle-hq/shuttle/commit/451387be36423ce4c8f041ec999c38f42ac1ea23))
- Project date ([#1141](https://github.com/shuttle-hq/shuttle/issues/1141)) - ([b8eec6a](https://github.com/shuttle-hq/shuttle/commit/b8eec6a512d454559d80eab230c54aa21e3c29b7))

### Miscellaneous Tasks

- *(container)* Use pre-installed cargo-chef image ([#1148](https://github.com/shuttle-hq/shuttle/issues/1148)) - ([af3d46a](https://github.com/shuttle-hq/shuttle/commit/af3d46aed8527216ecbc7996291537829c65f2a4))
- V0.24.0 ([#1153](https://github.com/shuttle-hq/shuttle/issues/1153)) - ([5fe4f5c](https://github.com/shuttle-hq/shuttle/commit/5fe4f5c5f7cebd73e79c54754fc2942f885616e3))
- Gateway v0.23.1 ([#1138](https://github.com/shuttle-hq/shuttle/issues/1138)) - ([f97443d](https://github.com/shuttle-hq/shuttle/commit/f97443dac07f3647c766cfea8f79047f564eef55))
- Upgrade mac binary build machine to m1 ([#1136](https://github.com/shuttle-hq/shuttle/issues/1136)) - ([5540eed](https://github.com/shuttle-hq/shuttle/commit/5540eed2f8647edaa4fb7c6da0168d40286a4c04))
- Always build images in release profile ([#1135](https://github.com/shuttle-hq/shuttle/issues/1135)) - ([8e4778f](https://github.com/shuttle-hq/shuttle/commit/8e4778fd97913edd403f10c5ec37fcb5dc2bfa03))

### Miscellaneous

- Fix clippy ([#1152](https://github.com/shuttle-hq/shuttle/issues/1152)) - ([251ff3a](https://github.com/shuttle-hq/shuttle/commit/251ff3a5c5196f45558c188fd5c87191391af7fb))
- Project_id label missing ([#1137](https://github.com/shuttle-hq/shuttle/issues/1137)) - ([9af8df3](https://github.com/shuttle-hq/shuttle/commit/9af8df39a137007e99867e5c081cf5d25e11f2e3))

## [0.23.0](https://github.com/shuttle-hq/shuttle/compare/v0.22.0..v0.23.0) - 2023-08-07

### Features

- *(runtime)* Add alpha runtime version check ([#1088](https://github.com/shuttle-hq/shuttle/issues/1088)) - ([f2cbc1b](https://github.com/shuttle-hq/shuttle/commit/f2cbc1b707426960ed4693d50420a4edfadcce6f))
- Gateway to start last deploy from idle project ([#1121](https://github.com/shuttle-hq/shuttle/issues/1121)) - ([4dfd65c](https://github.com/shuttle-hq/shuttle/commit/4dfd65c4bc27eeb69e246c3eaf557e1f8596e47a))

### Bug Fixes

- Install shuttle-next runtime in deployers ([#1127](https://github.com/shuttle-hq/shuttle/issues/1127)) - ([1a436ce](https://github.com/shuttle-hq/shuttle/commit/1a436cea790b38074271e8fca407342309c868e7))
- Add volume to mongo container ([#1126](https://github.com/shuttle-hq/shuttle/issues/1126)) - ([f770f8a](https://github.com/shuttle-hq/shuttle/commit/f770f8ad6ac33c8ab54280823f6f264f1305dead))

### Documentation

- Update readme, separate contributing and development docs ([#1124](https://github.com/shuttle-hq/shuttle/issues/1124)) - ([55ba155](https://github.com/shuttle-hq/shuttle/commit/55ba155cebd26ba902fa946a9d5992af7395fb74))

### Miscellaneous Tasks

- V0.23.0 ([#1133](https://github.com/shuttle-hq/shuttle/issues/1133)) - ([609411c](https://github.com/shuttle-hq/shuttle/commit/609411c2d98e52f20d24830aa527598a4347dc9f))
- Separate jobs for faster ci verdict ([#1130](https://github.com/shuttle-hq/shuttle/issues/1130)) - ([509e57b](https://github.com/shuttle-hq/shuttle/commit/509e57baf0d53434a0a89ca62e92447b19d55303))

### Miscellaneous

- Sync resource-recorder with persistence ([#1101](https://github.com/shuttle-hq/shuttle/issues/1101)) - ([c055cae](https://github.com/shuttle-hq/shuttle/commit/c055cae5d3d3780a352abc15f570040d4e0e2547))
- Add warning for api url arg ([#1128](https://github.com/shuttle-hq/shuttle/issues/1128)) - ([2f5ec20](https://github.com/shuttle-hq/shuttle/commit/2f5ec208012a206a6e68cec693eb8ba10ebe97f4))

## [0.22.0](https://github.com/shuttle-hq/shuttle/compare/v0.21.0..v0.22.0) - 2023-08-02

### Features

- Handling regular signals sent to cargo-shuttle on Windows ([#1077](https://github.com/shuttle-hq/shuttle/issues/1077)) - ([c3c5d4c](https://github.com/shuttle-hq/shuttle/commit/c3c5d4c16615b3c6d74ff6b565023b804a1ebbdf))
- Add a route to deployer to start a past deployment ([#1115](https://github.com/shuttle-hq/shuttle/issues/1115)) - ([39048ed](https://github.com/shuttle-hq/shuttle/commit/39048ed452bffc9664555693e489bb139ac51533))
- Expand gateways args to receive the key for the machine user ([#1114](https://github.com/shuttle-hq/shuttle/issues/1114)) - ([c8ccc87](https://github.com/shuttle-hq/shuttle/commit/c8ccc872c37c2e11d7e848917baaf28152e5a31e))
- Add deployer tier to auth ([#1111](https://github.com/shuttle-hq/shuttle/issues/1111)) - ([abfaab4](https://github.com/shuttle-hq/shuttle/commit/abfaab4a6e126182dc365612ed6ddc804249d62d))
- Honeycomb for local runs ([#1100](https://github.com/shuttle-hq/shuttle/issues/1100)) - ([cfcafde](https://github.com/shuttle-hq/shuttle/commit/cfcafde0e05611012bc6d825a28200bebb1a17dc))
- Add a project_id to gateway ([#1091](https://github.com/shuttle-hq/shuttle/issues/1091)) - ([c59483a](https://github.com/shuttle-hq/shuttle/commit/c59483adce97df7ad63292c79d02e0bfe880776a))
- Resource recorder ([#1084](https://github.com/shuttle-hq/shuttle/issues/1084)) - ([af2fcbc](https://github.com/shuttle-hq/shuttle/commit/af2fcbcabfea1cf5648a1530af2617b044369844))

### Bug Fixes

- *(codegen)* Allow `main` function name, prevent clippy warning ([#1098](https://github.com/shuttle-hq/shuttle/issues/1098)) - ([4de2c37](https://github.com/shuttle-hq/shuttle/commit/4de2c373506b81d6f39a5aee531f410af00d3032))
- Update information in README ([#1087](https://github.com/shuttle-hq/shuttle/issues/1087)) - ([0166bd5](https://github.com/shuttle-hq/shuttle/commit/0166bd5d7a23ef7af9bdcdd197ef63e1e5600b5a))

### Miscellaneous Tasks

- Bump sqlx in resources ([#1117](https://github.com/shuttle-hq/shuttle/issues/1117)) - ([4edb38b](https://github.com/shuttle-hq/shuttle/commit/4edb38b76e8cf15d126e0e857b117181f74c3804))
- Fix Actix Web typos ([#1093](https://github.com/shuttle-hq/shuttle/issues/1093)) - ([894d84e](https://github.com/shuttle-hq/shuttle/commit/894d84e37e4368bdad851428febd23969cf0ed15))
- Set HONEYCOMB env variable ([#1109](https://github.com/shuttle-hq/shuttle/issues/1109)) - ([1767982](https://github.com/shuttle-hq/shuttle/commit/176798283a5004863c951a0733bbab78d236de28))

### Miscellaneous

- Copy the certificate ([#1123](https://github.com/shuttle-hq/shuttle/issues/1123)) - ([0a78f36](https://github.com/shuttle-hq/shuttle/commit/0a78f369931a459b46c234d5f7af7950c400efaa))
- Chore/v0.22.0 ([#1119](https://github.com/shuttle-hq/shuttle/issues/1119)) - ([bc38d36](https://github.com/shuttle-hq/shuttle/commit/bc38d3644c4bf841b3e74355cf9e534565a416c2))
- Chore/bump sqlx ([#1118](https://github.com/shuttle-hq/shuttle/issues/1118)) - ([9d12b68](https://github.com/shuttle-hq/shuttle/commit/9d12b689e23c78a7b243941fa9fee3fcb1f31551))
- Fixed runtime logs receiving ([#1108](https://github.com/shuttle-hq/shuttle/issues/1108)) - ([1a400be](https://github.com/shuttle-hq/shuttle/commit/1a400be6017213180fc40beb11454a983e4caa90))
- Add LD_LIBRARY_PATH in docker-compose ([#1105](https://github.com/shuttle-hq/shuttle/issues/1105)) - ([53d9c25](https://github.com/shuttle-hq/shuttle/commit/53d9c25bf3861a7437557d306773d601b7eeff2e))
- Update the docker compose file ([#1092](https://github.com/shuttle-hq/shuttle/issues/1092)) - ([129ad74](https://github.com/shuttle-hq/shuttle/commit/129ad74700311238a4e0a839e5f66a5ad79abfd4))
- Added the missing support for creating custom resources ([#1082](https://github.com/shuttle-hq/shuttle/issues/1082)) - ([772805d](https://github.com/shuttle-hq/shuttle/commit/772805d1537839c32851e872661a9b50c508cb3b))

## [0.21.0](https://github.com/shuttle-hq/shuttle/compare/v0.20.0..v0.21.0) - 2023-07-10

### Bug Fixes

- Broken gateway pagination and status check tests ([#1075](https://github.com/shuttle-hq/shuttle/issues/1075)) - ([488c417](https://github.com/shuttle-hq/shuttle/commit/488c4171c9795d7c9764d21557f3755dcf157869))
- Rds deploy crash ([#1068](https://github.com/shuttle-hq/shuttle/issues/1068)) - ([d3aafff](https://github.com/shuttle-hq/shuttle/commit/d3aafff84d44daee7aa50b2974427a7bec6f776c))
- Remove md that is incompatible with docs ([#1063](https://github.com/shuttle-hq/shuttle/issues/1063)) - ([9ee11ed](https://github.com/shuttle-hq/shuttle/commit/9ee11ed054cfad3968b152462cdb958d11f0bce8))
- Bash bug in containerfile ([#1060](https://github.com/shuttle-hq/shuttle/issues/1060)) - ([c081d85](https://github.com/shuttle-hq/shuttle/commit/c081d85327f28d738633557583f978e4f060a48f))

### Miscellaneous Tasks

- V0.21.0 ([#1078](https://github.com/shuttle-hq/shuttle/issues/1078)) - ([c334a1c](https://github.com/shuttle-hq/shuttle/commit/c334a1c08cf82bc998ea6413edbbd3b969566373))
- Add --allow-dirty to command hint at end of init ([#1076](https://github.com/shuttle-hq/shuttle/issues/1076)) - ([b36ce58](https://github.com/shuttle-hq/shuttle/commit/b36ce583bbd8670e6137d594cc809b485a92555f))
- Bump tower-sanitize-path ([#1074](https://github.com/shuttle-hq/shuttle/issues/1074)) - ([3e73c11](https://github.com/shuttle-hq/shuttle/commit/3e73c11334eb39aba609bd0f424143edd37b2d4d))
- Incorrect path to turso resource ([#1062](https://github.com/shuttle-hq/shuttle/issues/1062)) - ([cbb71c4](https://github.com/shuttle-hq/shuttle/commit/cbb71c42f0b34f09f87545d5d3bf1930f71bc5b1))
- Update git submodule path ([#1049](https://github.com/shuttle-hq/shuttle/issues/1049)) - ([cae4b1b](https://github.com/shuttle-hq/shuttle/commit/cae4b1b54cfcfc8d335b392e103aba540735f98c))
- Add turso to publish job ([#1059](https://github.com/shuttle-hq/shuttle/issues/1059)) - ([bbaef9f](https://github.com/shuttle-hq/shuttle/commit/bbaef9f78f506726c39e4cb6353ab53d5dd5d139))

### Miscellaneous

- Explain 413 error when a user tries to create a project larger than the limit ([#1070](https://github.com/shuttle-hq/shuttle/issues/1070)) - ([e604b55](https://github.com/shuttle-hq/shuttle/commit/e604b55349eb60a939db71e231b0f04e18b61f52))

## [0.20.0](https://github.com/shuttle-hq/shuttle/compare/v0.19.0..v0.20.0) - 2023-06-28

### Features

- *(resources)* Add support for turso client w/o provisioning ([#996](https://github.com/shuttle-hq/shuttle/issues/996)) - ([4ea9883](https://github.com/shuttle-hq/shuttle/commit/4ea988330584319fde8786390a87112ab823e2d6))
- Add new deployment metadata to table ([#987](https://github.com/shuttle-hq/shuttle/issues/987)) - ([fa8056a](https://github.com/shuttle-hq/shuttle/commit/fa8056a0515d144e45bfaea9f1ad6686345c43e0))

### Bug Fixes

- *(runtime)* Remove 2s startup sleep ([#1012](https://github.com/shuttle-hq/shuttle/issues/1012)) - ([e9906f5](https://github.com/shuttle-hq/shuttle/commit/e9906f50b1048f91ea47e25757559e27bf454121))
- Windows local run + log clarifications ([#1054](https://github.com/shuttle-hq/shuttle/issues/1054)) - ([63bcf8f](https://github.com/shuttle-hq/shuttle/commit/63bcf8fbaf76cc110a1c067a693619c2c905824b))
- Dockerfile and ci improvements ([#989](https://github.com/shuttle-hq/shuttle/issues/989)) - ([947d6a7](https://github.com/shuttle-hq/shuttle/commit/947d6a7d7e06bfd9282cc4f756c7daa0a5af9d38))
- Target directory from config, Windows .exe suffix ([#1039](https://github.com/shuttle-hq/shuttle/issues/1039)) - ([3d6ff56](https://github.com/shuttle-hq/shuttle/commit/3d6ff569b3560fa551f7103a5121b6908ce5a843))
- Don't deploy .git folder to save space ([#1036](https://github.com/shuttle-hq/shuttle/issues/1036)) - ([97f954a](https://github.com/shuttle-hq/shuttle/commit/97f954aab99bd7b54a54013feb11d99daf103fda))
- Increase body size limit for deploy ([#1031](https://github.com/shuttle-hq/shuttle/issues/1031)) - ([3b10128](https://github.com/shuttle-hq/shuttle/commit/3b10128a043aa6d08dca0537c5b80bb5d57cba52))
- Cargo-generate needs openssl ([#1023](https://github.com/shuttle-hq/shuttle/issues/1023)) - ([30e512c](https://github.com/shuttle-hq/shuttle/commit/30e512ceb7a1be86e89d4ebed181fff04e30afe8))
- Remove vendored-openssl from CI and broken axum test ([#1021](https://github.com/shuttle-hq/shuttle/issues/1021)) - ([5d34242](https://github.com/shuttle-hq/shuttle/commit/5d34242488962a7fa51c60f5240b898e9193194c))

### Miscellaneous Tasks

- *(shell.nix)* Add openssl package to the build dependencies ([#1040](https://github.com/shuttle-hq/shuttle/issues/1040)) - ([7075917](https://github.com/shuttle-hq/shuttle/commit/70759177648de50f1e407c3a95b8c37ace8864e9))
- Move codegen::main from service to runtime ([#1013](https://github.com/shuttle-hq/shuttle/issues/1013)) - ([9907349](https://github.com/shuttle-hq/shuttle/commit/990734987fa133a2f00c1bd422f59e9fcd6a8e00))
- Cargo-shuttle v0.19.1 ([#1037](https://github.com/shuttle-hq/shuttle/issues/1037)) - ([c0d48b5](https://github.com/shuttle-hq/shuttle/commit/c0d48b51d457e287bbc0539f8eb174f01cf87882))

### Miscellaneous

- *(common)* Format logs in correct local timezone ([#1032](https://github.com/shuttle-hq/shuttle/issues/1032)) - ([e770349](https://github.com/shuttle-hq/shuttle/commit/e7703493a665b135fabe65d9882efce7a7564234))
- Chore/v0.20.0 ([#1057](https://github.com/shuttle-hq/shuttle/issues/1057)) - ([bf0365a](https://github.com/shuttle-hq/shuttle/commit/bf0365a4c99ffbc6936c725ba8f9d813dfeff2c1))
- Status check includes info about auth & provisioner ([#1056](https://github.com/shuttle-hq/shuttle/issues/1056)) - ([59af379](https://github.com/shuttle-hq/shuttle/commit/59af3795b2942e39d1371c1b414beb14c9fa2bb3))
- Attempt at fixing sporadic failures of `shuttle-deployer` ([#980](https://github.com/shuttle-hq/shuttle/issues/980)) - ([9aef803](https://github.com/shuttle-hq/shuttle/commit/9aef803af40e684ce023e1dc5de12ba22ed1691b))

## [0.19.0](https://github.com/shuttle-hq/shuttle/compare/v0.18.0..v0.19.0) - 2023-06-20

### Features

- *(deployer)* Add more deployment metadata ([#943](https://github.com/shuttle-hq/shuttle/issues/943)) - ([6cb2cf2](https://github.com/shuttle-hq/shuttle/commit/6cb2cf20a57fea0cf6872e294726573141c5abef))
- *(secrets)* Implement into_iter for SecretStore ([#1006](https://github.com/shuttle-hq/shuttle/issues/1006)) - ([15bd0ae](https://github.com/shuttle-hq/shuttle/commit/15bd0aefdb4ff35596cf3e78fead73a4e49ad6ac))
- *(service)* Get rid of `cargo` dependency ([#922](https://github.com/shuttle-hq/shuttle/issues/922)) - ([7c01a73](https://github.com/shuttle-hq/shuttle/commit/7c01a7363eaa8d55cc684ed54ad21ea935802a3a))
- Shuttle init --from ([#984](https://github.com/shuttle-hq/shuttle/issues/984)) - ([73cf246](https://github.com/shuttle-hq/shuttle/commit/73cf24637e86f880d7fc11cc5222d3729ae14671))
- Use `cargo generate` instead of hardcoding examples source code ([#888](https://github.com/shuttle-hq/shuttle/issues/888)) - ([8bb05b6](https://github.com/shuttle-hq/shuttle/commit/8bb05b640a4d0607dfdebe58ccda7f9df49a8993))
- Switch from native-tls to rustls ([#879](https://github.com/shuttle-hq/shuttle/issues/879)) - ([dfa4950](https://github.com/shuttle-hq/shuttle/commit/dfa49502779e8cc2295749e204d1f24eff7e297c))

### Bug Fixes

- Mysql provisioning bug ([#1001](https://github.com/shuttle-hq/shuttle/issues/1001)) - ([5089c9c](https://github.com/shuttle-hq/shuttle/commit/5089c9c56fc3c350b3b99d4ae12811a0e08585b4))
- Remove auth login endpoint ([#1007](https://github.com/shuttle-hq/shuttle/issues/1007)) - ([b69e7ef](https://github.com/shuttle-hq/shuttle/commit/b69e7ef3ea7fafa967e9bb4f17adedf65f75689c))
- Re-add `--version` flag ([#998](https://github.com/shuttle-hq/shuttle/issues/998)) - ([c136fde](https://github.com/shuttle-hq/shuttle/commit/c136fde1892686255d4a8de9bfc586997ee84b5b))
- Logs --latest flipped order ([#982](https://github.com/shuttle-hq/shuttle/issues/982)) - ([085eb30](https://github.com/shuttle-hq/shuttle/commit/085eb306b553bd517e997afd01b532af18cd230e))

### Documentation

- Add sections on PR and reviews ([#1016](https://github.com/shuttle-hq/shuttle/issues/1016)) - ([beaddda](https://github.com/shuttle-hq/shuttle/commit/beaddda6699bc4fa07bc15ade6473e0522508fd2))

### Miscellaneous Tasks

- V0.19.0 ([#1019](https://github.com/shuttle-hq/shuttle/issues/1019)) - ([183a81e](https://github.com/shuttle-hq/shuttle/commit/183a81ee71f5e8180e6cc7d1ad81376b90a3f085))
- Bump pinned Rust to 1.70 ([#967](https://github.com/shuttle-hq/shuttle/issues/967)) - ([316b7a3](https://github.com/shuttle-hq/shuttle/commit/316b7a3b2a9ff3b5a60507871c35f138e565c357))

### Miscellaneous

- Provide better error message ([#993](https://github.com/shuttle-hq/shuttle/issues/993)) - ([375b616](https://github.com/shuttle-hq/shuttle/commit/375b6161d4b2df77128a030398bc224e3a613c90))
- Add commands cookbook to cargo-shuttle --help ([#985](https://github.com/shuttle-hq/shuttle/issues/985)) - ([1b47611](https://github.com/shuttle-hq/shuttle/commit/1b476119e8557f38b8e74c61b8e1196eb6bbe519))

## [0.18.0](https://github.com/shuttle-hq/shuttle/compare/v0.17.0..v0.18.0) - 2023-06-05

### Features

- *(gateway, cargo-shuttle)* Implement pagination for project list ([#862](https://github.com/shuttle-hq/shuttle/issues/862)) - ([b7e5e3b](https://github.com/shuttle-hq/shuttle/commit/b7e5e3b44a19e8cab9b02d73a3e066af5ac6f174))
- Pre-installed build environment in deployer ([#960](https://github.com/shuttle-hq/shuttle/issues/960)) - ([795ec74](https://github.com/shuttle-hq/shuttle/commit/795ec744fa4487cef505495582f0fee30b5ce409))
- Feat(shuttle-axum) Make AxumService generic to be able to use axum::State with it ([#924](https://github.com/shuttle-hq/shuttle/issues/924)) - ([e6ade25](https://github.com/shuttle-hq/shuttle/commit/e6ade25efbca897a732ee3bb4f7de285b8e16d50))

### Bug Fixes

- *(prod)* Unstable AWS creds clashed with prod ([#970](https://github.com/shuttle-hq/shuttle/issues/970)) - ([eb1ea84](https://github.com/shuttle-hq/shuttle/commit/eb1ea845104766a9192651dc0b5661ed153fa0f3))
- Remove cargo-sort from CONTRIBUTING.md ([#966](https://github.com/shuttle-hq/shuttle/issues/966)) - ([d35db19](https://github.com/shuttle-hq/shuttle/commit/d35db191434dec6b2db9ce0a84018bb7fd72f964))
- Ignore span logs below WARN ([#958](https://github.com/shuttle-hq/shuttle/issues/958)) - ([f68f0f5](https://github.com/shuttle-hq/shuttle/commit/f68f0f51c11ca3a47cc29050018f974376d3d5b0))
- Crossterm/comfytable conflict ([#959](https://github.com/shuttle-hq/shuttle/issues/959)) - ([078aec2](https://github.com/shuttle-hq/shuttle/commit/078aec2c06d0ab769831474b10cfff20c76869cf))
- Log files packed in archive ([#931](https://github.com/shuttle-hq/shuttle/issues/931)) - ([322b7f6](https://github.com/shuttle-hq/shuttle/commit/322b7f64442ae6e33795cb7a5d57d8d54eb6dfbd))
- --name was ignored when not running from cargo folder ([#929](https://github.com/shuttle-hq/shuttle/issues/929)) - ([622950f](https://github.com/shuttle-hq/shuttle/commit/622950f009c58a5842f0c443df3a1cd7e8674eff))

### Refactor

- Un-tangle crossterm/comfytable ([#961](https://github.com/shuttle-hq/shuttle/issues/961)) - ([b358523](https://github.com/shuttle-hq/shuttle/commit/b358523e8bf9c101bcb4d5e2fc028d163aacf4b5))
- Sanitize all path on the user's proxy ([#946](https://github.com/shuttle-hq/shuttle/issues/946)) - ([fa0e4e3](https://github.com/shuttle-hq/shuttle/commit/fa0e4e3f018b51a3bec59b876737abe38a840ef7))

### Documentation

- Update links and commands ([#948](https://github.com/shuttle-hq/shuttle/issues/948)) - ([05d0f1b](https://github.com/shuttle-hq/shuttle/commit/05d0f1b5678350cdcf25b84fd678ec9c67bc4399))

### Miscellaneous Tasks

- V0.18.0 ([#972](https://github.com/shuttle-hq/shuttle/issues/972)) - ([f2a3c4d](https://github.com/shuttle-hq/shuttle/commit/f2a3c4d56799295ef17233a7115b8d9f97889575))
- Bump otel crates and remove protoc dep ([#956](https://github.com/shuttle-hq/shuttle/issues/956)) - ([fc91472](https://github.com/shuttle-hq/shuttle/commit/fc914727f367a2523db63bd3d6c3d2ede75514d7))
- Update Cargo.lock ([#942](https://github.com/shuttle-hq/shuttle/issues/942)) - ([7eefced](https://github.com/shuttle-hq/shuttle/commit/7eefced4c6db1efc5eddca96caaf31a17c5d63d4))

### Miscellaneous

- Add helpful error if port cannot be used ([#950](https://github.com/shuttle-hq/shuttle/issues/950)) - ([1fc3667](https://github.com/shuttle-hq/shuttle/commit/1fc3667acf01c633c665db2e414b22fbbfd5f8b9))
- Update GitHub templates ([#945](https://github.com/shuttle-hq/shuttle/issues/945)) - ([2cbb1c2](https://github.com/shuttle-hq/shuttle/commit/2cbb1c28203291f884044aec091225a7daa1cd41))

## [0.17.0](https://github.com/shuttle-hq/shuttle/compare/v0.16.0..v0.17.0) - 2023-05-22

### Features

- *(Makefile)* Add option to disable --detach on make up ([#878](https://github.com/shuttle-hq/shuttle/issues/878)) - ([ab12fdd](https://github.com/shuttle-hq/shuttle/commit/ab12fdd97f1337b2939bfea1d2581ef2f8235460))
- *(cargo-shuttle)* Log reconnects and improved error messages ([#853](https://github.com/shuttle-hq/shuttle/issues/853)) - ([cba9c48](https://github.com/shuttle-hq/shuttle/commit/cba9c48b465a38ce6200a9c5f7b983c432cfe206))
- *(runtime)* Remove dependency on clap ([#822](https://github.com/shuttle-hq/shuttle/issues/822)) - ([5f0874c](https://github.com/shuttle-hq/shuttle/commit/5f0874c966a6ec9b5b9c9a0f3ca1ed90e56460ca))
- Allow resetting a user's API-key ([#857](https://github.com/shuttle-hq/shuttle/issues/857)) - ([4d2c0c2](https://github.com/shuttle-hq/shuttle/commit/4d2c0c2ba122f5e36d6c649ddecf6e2671a008a6))
- Show output of failed tests ([#907](https://github.com/shuttle-hq/shuttle/issues/907)) - ([71c9280](https://github.com/shuttle-hq/shuttle/commit/71c928088f7ac141a67d08e26ace82ea01cb253f))

### Bug Fixes

- *(gateway)* Handle certificate expiration as well ([#932](https://github.com/shuttle-hq/shuttle/issues/932)) - ([51b6bc5](https://github.com/shuttle-hq/shuttle/commit/51b6bc5d081928c2740782f758682b6701817ab0))
- Deployment state shown as running on startup crash ([#919](https://github.com/shuttle-hq/shuttle/issues/919)) - ([53bf341](https://github.com/shuttle-hq/shuttle/commit/53bf34147497276468fa05cdbb53c4fcae4bac2c))
- Set correct admin scopes in scopebuilder ([#899](https://github.com/shuttle-hq/shuttle/issues/899)) - ([7b90021](https://github.com/shuttle-hq/shuttle/commit/7b90021e1d9523ddd6fcdbb86c38227713bd9313))
- Revert addition of apikey to auth ([#886](https://github.com/shuttle-hq/shuttle/issues/886)) - ([7054e6a](https://github.com/shuttle-hq/shuttle/commit/7054e6ae34d114b8c0dd33d23f671aad66e36445))

### Documentation

- Contributing updates ([#918](https://github.com/shuttle-hq/shuttle/issues/918)) - ([7a20b70](https://github.com/shuttle-hq/shuttle/commit/7a20b70ae47366aed3fb6ebc1a2bf8dacb9c13fc))
- Add installation instructions for Arch Linux ([#902](https://github.com/shuttle-hq/shuttle/issues/902)) - ([d056bcd](https://github.com/shuttle-hq/shuttle/commit/d056bcd6d0e3976413c56af297760a7938327fc3))

### Miscellaneous Tasks

- Production deployment automation ([#920](https://github.com/shuttle-hq/shuttle/issues/920)) - ([959dab1](https://github.com/shuttle-hq/shuttle/commit/959dab10f91fe773396a18bf371e6a80c472c938))
- Add windows qa ([#812](https://github.com/shuttle-hq/shuttle/issues/812)) - ([d0b2f32](https://github.com/shuttle-hq/shuttle/commit/d0b2f32cdfeba02b7fb145c11d8df57d0ad9a3ba))
- Promote hyper-reverse-proxy to a workspace dependency ([#921](https://github.com/shuttle-hq/shuttle/issues/921)) - ([b5f35f4](https://github.com/shuttle-hq/shuttle/commit/b5f35f46a223fb9298e2feba4ff2d48099a38c41))
- Upgrade salvo in shuttle-salvo ([#901](https://github.com/shuttle-hq/shuttle/issues/901)) - ([7009284](https://github.com/shuttle-hq/shuttle/commit/7009284159e7bff8916a8acc27e5a175a0265dd9))
- Release automation on unstable ([#816](https://github.com/shuttle-hq/shuttle/issues/816)) - ([e1202b4](https://github.com/shuttle-hq/shuttle/commit/e1202b4ee771fd48d3c87b0c3d56dd0bda0ed546))
- Reimplemented JwtAuthentication with struct-based Future. ([#868](https://github.com/shuttle-hq/shuttle/issues/868)) - ([d4322be](https://github.com/shuttle-hq/shuttle/commit/d4322bec44c84f0d75d39c785a158206df19d1ad))
- Bump common to 0.16.2 ([#900](https://github.com/shuttle-hq/shuttle/issues/900)) - ([0a97df1](https://github.com/shuttle-hq/shuttle/commit/0a97df1c755df9337a4b4b24d9e488620be46e7a))
- Update aws crates ([#897](https://github.com/shuttle-hq/shuttle/issues/897)) - ([d4a8c99](https://github.com/shuttle-hq/shuttle/commit/d4a8c99452d214f0e035e0494a9e70967fca1c26))
- Add option to use rustls instead of native-tls in `shuttle-shared-db` ([#870](https://github.com/shuttle-hq/shuttle/issues/870)) - ([04407e8](https://github.com/shuttle-hq/shuttle/commit/04407e8827ec750f3ffbe6aa82beefaf0f508cac))

### Revert

- Revert #886 ([#887](https://github.com/shuttle-hq/shuttle/issues/887)) - ([b861afd](https://github.com/shuttle-hq/shuttle/commit/b861afd0c58192c11a058f59328b5aa816a89828))

### Miscellaneous

- *(prod)* Add protoc dependency and fix the crates order ([#938](https://github.com/shuttle-hq/shuttle/issues/938)) - ([5b9885c](https://github.com/shuttle-hq/shuttle/commit/5b9885ce5dfdf29b1bc60ad26c63aa3abc58eb99))
- *(prod)* Fix the missing line break escape ([#937](https://github.com/shuttle-hq/shuttle/issues/937)) - ([7d15a5f](https://github.com/shuttle-hq/shuttle/commit/7d15a5f395353a052bdda1d007d8b361b42aef34))
- *(prod)* Gate against local crates.io patch ([#936](https://github.com/shuttle-hq/shuttle/issues/936)) - ([d1c6ed1](https://github.com/shuttle-hq/shuttle/commit/d1c6ed13c52b02a686226ef60c1d90aab3d56d4a))
- Chore/0.17.0 ([#934](https://github.com/shuttle-hq/shuttle/issues/934)) - ([6c1de7e](https://github.com/shuttle-hq/shuttle/commit/6c1de7ec23f70f20ac6f6e10157799c85a59d20f))
- Suggest next logical command ([#915](https://github.com/shuttle-hq/shuttle/issues/915)) - ([d9e4255](https://github.com/shuttle-hq/shuttle/commit/d9e4255fcaeddb3f0767d2698c72a42cf249d1ce))
- Update README.md - ([3ca63c6](https://github.com/shuttle-hq/shuttle/commit/3ca63c6ea383714c16b4497108f495d217d1b4d7))
- Update README.md - ([a66a476](https://github.com/shuttle-hq/shuttle/commit/a66a476a0d24b56e7e0c2098ba4708acd7b8e535))
- Update/syn 2.0 ([#880](https://github.com/shuttle-hq/shuttle/issues/880)) - ([8af0b46](https://github.com/shuttle-hq/shuttle/commit/8af0b4636267f5bb547300956607527de6a29ae1))
- Match doc links with Shuttle Service current doc url ([#885](https://github.com/shuttle-hq/shuttle/issues/885)) - ([58068ac](https://github.com/shuttle-hq/shuttle/commit/58068ace635db37e1f027c598b8fa194c62033c9))

## [0.16.0](https://github.com/shuttle-hq/shuttle/compare/v0.15.0..v0.16.0) - 2023-05-08

### Features

- Add on_new_span impl to runtime Logger ([#864](https://github.com/shuttle-hq/shuttle/issues/864)) - ([92d7f7b](https://github.com/shuttle-hq/shuttle/commit/92d7f7bd339e950f06be7ac92f5c556273b566cb))
- Remove /hello from tests/ci ([#863](https://github.com/shuttle-hq/shuttle/issues/863)) - ([3a6b378](https://github.com/shuttle-hq/shuttle/commit/3a6b378a3470c0e5a5337b5e63a9bb3fa9eb5cde))
- ApiKey newtype to ensure key is always valid format ([#835](https://github.com/shuttle-hq/shuttle/issues/835)) - ([fae2733](https://github.com/shuttle-hq/shuttle/commit/fae27332bede8a0767cbbca7b6e9a29461a965bd))
- Refactor deployer to run locally without auth ([#810](https://github.com/shuttle-hq/shuttle/issues/810)) - ([05d9266](https://github.com/shuttle-hq/shuttle/commit/05d9266a785f3b4d790979f94b71fb60ef1ea97b))

### Bug Fixes

- Some panic messages get lost ([#854](https://github.com/shuttle-hq/shuttle/issues/854)) - ([991a579](https://github.com/shuttle-hq/shuttle/commit/991a579be26aefd2923fd86dc5c480f0266083f5))
- `make test` ([#858](https://github.com/shuttle-hq/shuttle/issues/858)) - ([05472fc](https://github.com/shuttle-hq/shuttle/commit/05472fc004cc72138c3adb2fcf9f970101f48705))
- Shuttle init --template, reorder subcommands, fix bugs ([#792](https://github.com/shuttle-hq/shuttle/issues/792)) - ([b1e5448](https://github.com/shuttle-hq/shuttle/commit/b1e5448b933282220fb2dc41f81cfa41ccbd0564))
- Remove unused project list filtering ([#832](https://github.com/shuttle-hq/shuttle/issues/832)) - ([c34b0f2](https://github.com/shuttle-hq/shuttle/commit/c34b0f2b3c953ad7bce61ee2dae612cf238611de))
- Disable docker QA ([#830](https://github.com/shuttle-hq/shuttle/issues/830)) - ([18108fb](https://github.com/shuttle-hq/shuttle/commit/18108fb912ce500695dd6ec8af3f81ad5dc091b1))
- Wasm qa casing ([#828](https://github.com/shuttle-hq/shuttle/issues/828)) - ([1106c5f](https://github.com/shuttle-hq/shuttle/commit/1106c5fccf88c3780b5b311864f52ba72f392160))

### Documentation

- Document how to generate protofiles ([#836](https://github.com/shuttle-hq/shuttle/issues/836)) - ([dbbc2a7](https://github.com/shuttle-hq/shuttle/commit/dbbc2a763d5c09d45933c2b82231cd0e3531a6d5))

### Miscellaneous Tasks

- V0.16.0 ([#881](https://github.com/shuttle-hq/shuttle/issues/881)) - ([62a21c1](https://github.com/shuttle-hq/shuttle/commit/62a21c17d035fb62586ca2d0c786ccdca0d5d12c))
- Add `.editorconfig` ([#855](https://github.com/shuttle-hq/shuttle/issues/855)) - ([a4bf52f](https://github.com/shuttle-hq/shuttle/commit/a4bf52fb80e3d908f539bee7f015b7920ab818ad))
- Download sccache instead of compiling it ([#859](https://github.com/shuttle-hq/shuttle/issues/859)) - ([a59216c](https://github.com/shuttle-hq/shuttle/commit/a59216c960dabf311a8756a24970d948621e4ea4))

### Miscellaneous

- *(docker)* Change default provisioner port to 3000 ([#852](https://github.com/shuttle-hq/shuttle/issues/852)) - ([0ec6509](https://github.com/shuttle-hq/shuttle/commit/0ec6509cf7bb0f4e3b4df4305167ff20fd16d7db))
- Rename examples to shuttle-examples ([#871](https://github.com/shuttle-hq/shuttle/issues/871)) - ([775b577](https://github.com/shuttle-hq/shuttle/commit/775b57784d15d950b77194818fd118ae89886cf6))
- Typo-fix - ([d81c201](https://github.com/shuttle-hq/shuttle/commit/d81c201952bfbbeaad9b021166a4a392a1936392))
- Added shuttle console sneak peek - ([2747869](https://github.com/shuttle-hq/shuttle/commit/2747869a5ddfc69ad54f415d4fad25810190e28e))
- Add star gif - ([7905dfe](https://github.com/shuttle-hq/shuttle/commit/7905dfe4a69e6ab59ab2bdc0044cfd41cc3d39e5))

## [0.15.0](https://github.com/shuttle-hq/shuttle/compare/v0.14.0..v0.15.0) - 2023-04-27

### Features

- Ensure interactive login API key is valid format ([#797](https://github.com/shuttle-hq/shuttle/issues/797)) - ([408a4c5](https://github.com/shuttle-hq/shuttle/commit/408a4c59b578f73584c5bdb1df02fc14e1009083))
- Improve deployer 404 messages ([#796](https://github.com/shuttle-hq/shuttle/issues/796)) - ([4ce62aa](https://github.com/shuttle-hq/shuttle/commit/4ce62aa5eeff1d4673c7d60b57734a636ee30c46))
- Add `cargo shuttle logs --latest` ([#799](https://github.com/shuttle-hq/shuttle/issues/799)) - ([5bdd892](https://github.com/shuttle-hq/shuttle/commit/5bdd89248d4fe5944422b024bc0c78f28e98f33f))
- Add algora shields to readme ([#793](https://github.com/shuttle-hq/shuttle/issues/793)) - ([89e50fa](https://github.com/shuttle-hq/shuttle/commit/89e50fa84ae4052230131f96c4ca04de3f02d1ef))

### Bug Fixes

- *(common)* Feature flagged utoipa dependency ([#817](https://github.com/shuttle-hq/shuttle/issues/817)) - ([9ddd3d5](https://github.com/shuttle-hq/shuttle/commit/9ddd3d506315474cedefbd8f1d57d722f256b53a))
- Apply admin layer to admin routes only ([#818](https://github.com/shuttle-hq/shuttle/issues/818)) - ([f225da0](https://github.com/shuttle-hq/shuttle/commit/f225da08135af768cb38b8137627e7e269b99202))
- Stop logging the full loadresponse ([#814](https://github.com/shuttle-hq/shuttle/issues/814)) - ([68aec3e](https://github.com/shuttle-hq/shuttle/commit/68aec3e11d76ed3cd7024c218398bc10cb22a46b))
- Minor development bug fixes ([#806](https://github.com/shuttle-hq/shuttle/issues/806)) - ([c596e46](https://github.com/shuttle-hq/shuttle/commit/c596e4688d99bb4c4828c237409cb08d6d02502a))

### Documentation

- Add note about init bug to readme ([#824](https://github.com/shuttle-hq/shuttle/issues/824)) - ([9542444](https://github.com/shuttle-hq/shuttle/commit/9542444ce0477522d3cb94a17519fdf2d65f3b20))
- Clarify shuttle_static_folder will not be adjacent to executable ([#803](https://github.com/shuttle-hq/shuttle/issues/803)) - ([b048d9c](https://github.com/shuttle-hq/shuttle/commit/b048d9cf11f0b73c73e1c5089029ac6d99435c4f))

### Miscellaneous Tasks

- Protoc removal ([#826](https://github.com/shuttle-hq/shuttle/issues/826)) - ([b626bf0](https://github.com/shuttle-hq/shuttle/commit/b626bf0874e9aabc146f0c99d7ba391d61957a38))
- V0.15.0 ([#820](https://github.com/shuttle-hq/shuttle/issues/820)) - ([7d90241](https://github.com/shuttle-hq/shuttle/commit/7d90241abed69abd059a161d5e503d35b9dc580e))
- Fix windows binary build ([#825](https://github.com/shuttle-hq/shuttle/issues/825)) - ([d59bffb](https://github.com/shuttle-hq/shuttle/commit/d59bffb186dc0a153b466224e775043e973e7ff2))
- Bump otel crates to remove protoc requirement ([#802](https://github.com/shuttle-hq/shuttle/issues/802)) - ([70b9838](https://github.com/shuttle-hq/shuttle/commit/70b98385bea9e82e725139ed01b10e97f83f63d3))
- Add mac qa ([#801](https://github.com/shuttle-hq/shuttle/issues/801)) - ([838bc3d](https://github.com/shuttle-hq/shuttle/commit/838bc3d965bfd2ccdfe653e69b287c136d7c00b3))
- Linux QA ([#800](https://github.com/shuttle-hq/shuttle/issues/800)) - ([44c1299](https://github.com/shuttle-hq/shuttle/commit/44c1299bba60a224e634ddbf1d64cc264604c178))

### Miscellaneous

- Separated unix from windows local_run ([#823](https://github.com/shuttle-hq/shuttle/issues/823)) - ([f97bdb4](https://github.com/shuttle-hq/shuttle/commit/f97bdb43181930b69a5a4773d23dab66147589eb))
- Fix address in use error when service panicked in a previous run ([#805](https://github.com/shuttle-hq/shuttle/issues/805)) - ([458cf25](https://github.com/shuttle-hq/shuttle/commit/458cf2552ca4556954baa9f1287d05121b1310fb))
- Deployer & gateway: added OpenAPI docs ([#794](https://github.com/shuttle-hq/shuttle/issues/794)) - ([66c1948](https://github.com/shuttle-hq/shuttle/commit/66c194895e012b02c00cc3a343f522e76560ba35))

## [0.14.0](https://github.com/shuttle-hq/shuttle/compare/v0.13.0..v0.14.0) - 2023-04-12

### Features

- Use relative url for examples submodule to allow cloning via git or https ([#776](https://github.com/shuttle-hq/shuttle/issues/776)) - ([b0390e1](https://github.com/shuttle-hq/shuttle/commit/b0390e1f22bba137f1ebec8666ae9f847dd0f493))
- Rename `project new/rm` to `start/stop`, add `restart` + other args fixes ([#771](https://github.com/shuttle-hq/shuttle/issues/771)) - ([629ac8c](https://github.com/shuttle-hq/shuttle/commit/629ac8c6a5e46da14c99a5fe130054852fa80934))
- Start all services in a workspace for local runs ([#772](https://github.com/shuttle-hq/shuttle/issues/772)) - ([515bd3f](https://github.com/shuttle-hq/shuttle/commit/515bd3f353fd7fad0cbda93519be273a0953c215))
- Use sparse registry in deployers ([#773](https://github.com/shuttle-hq/shuttle/issues/773)) - ([b37c9ef](https://github.com/shuttle-hq/shuttle/commit/b37c9ef9151043b81a05ee7fac9ac8cd2dcf3e19))
- Compile an entire workspace ([#767](https://github.com/shuttle-hq/shuttle/issues/767)) - ([36edf0a](https://github.com/shuttle-hq/shuttle/commit/36edf0a38632f1dc5762418d02bcdd4fe942e1e4))
- Commit generated proto files ([#753](https://github.com/shuttle-hq/shuttle/issues/753)) - ([915a53c](https://github.com/shuttle-hq/shuttle/commit/915a53c50ba57962bde2f1c5061ce99573dca0b7))

### Bug Fixes

- Cargo shuttle integration tests, project cmd renaming ([#789](https://github.com/shuttle-hq/shuttle/issues/789)) - ([1db4ae7](https://github.com/shuttle-hq/shuttle/commit/1db4ae70937868425260978978a9cea336839459))
- Revert use of portpicker for local run ([#787](https://github.com/shuttle-hq/shuttle/issues/787)) - ([4f34b49](https://github.com/shuttle-hq/shuttle/commit/4f34b4943561f3fe837283961b5cf27cb4e060ef))
- Secrets not archived in workspace crates ([#785](https://github.com/shuttle-hq/shuttle/issues/785)) - ([6312e53](https://github.com/shuttle-hq/shuttle/commit/6312e53f1fa18b4ec60609a2bc87c66e91bf8c57))
- Is_dirty path bug on windows ([#783](https://github.com/shuttle-hq/shuttle/issues/783)) - ([d54e14c](https://github.com/shuttle-hq/shuttle/commit/d54e14c8e40ad91ff9b4664b0d3516846e3d34d7))
- Timing of deployment status and local run printouts ([#744](https://github.com/shuttle-hq/shuttle/issues/744)) - ([07be36c](https://github.com/shuttle-hq/shuttle/commit/07be36c15733bc17a26dbe2a26d26e2d7c6d1ca6))
- Windows path canonicalization bug in static folder ([#762](https://github.com/shuttle-hq/shuttle/issues/762)) - ([44068cb](https://github.com/shuttle-hq/shuttle/commit/44068cbde02a611200426a7d74f0b6ec600d4dfa))

### Refactor

- Get the static folder name from the ([#780](https://github.com/shuttle-hq/shuttle/issues/780)) - ([3ce6144](https://github.com/shuttle-hq/shuttle/commit/3ce6144dc9275b60425a2f3ac7c0522782328695))
- Enable exhaustiveness check of command matching ([#768](https://github.com/shuttle-hq/shuttle/issues/768)) - ([126fe5c](https://github.com/shuttle-hq/shuttle/commit/126fe5ca4cade8eacda0942cc72559215722bcf2))
- Getting project name ([#774](https://github.com/shuttle-hq/shuttle/issues/774)) - ([70457b0](https://github.com/shuttle-hq/shuttle/commit/70457b017d9e8bdd9354a9e86836c34e4e6e0bd3))

### Documentation

- Update deployer local run guide ([#784](https://github.com/shuttle-hq/shuttle/issues/784)) - ([2378245](https://github.com/shuttle-hq/shuttle/commit/237824561fd95c307a70e7c6478985fd7f217fa2))

### Testing

- Make provisioner test deterministic ([#770](https://github.com/shuttle-hq/shuttle/issues/770)) - ([ba241c5](https://github.com/shuttle-hq/shuttle/commit/ba241c5d25b2fa101f20c5b15fed78d995aca74b))

### Miscellaneous Tasks

- V0.14.0 ([#788](https://github.com/shuttle-hq/shuttle/issues/788)) - ([dfacd2f](https://github.com/shuttle-hq/shuttle/commit/dfacd2fbeda332cd0865aea8c1a47ed2774e2ffa))
- Update Makefile for better Podman support ([#724](https://github.com/shuttle-hq/shuttle/issues/724)) - ([6ec660f](https://github.com/shuttle-hq/shuttle/commit/6ec660fe453d0157e9939d57859399c02f13a58f))
- Update bollard to v0.14.0 ([#722](https://github.com/shuttle-hq/shuttle/issues/722)) - ([bdccfb1](https://github.com/shuttle-hq/shuttle/commit/bdccfb1099f7cef005e939dfa7c5de5170072790))
- Bump static folder to 0.13.1 ([#764](https://github.com/shuttle-hq/shuttle/issues/764)) - ([51bd3d8](https://github.com/shuttle-hq/shuttle/commit/51bd3d89c5c49af1bf97c23772bd5f3846b5c3fd))

### Miscellaneous

- Infer environment based on storage type ([#786](https://github.com/shuttle-hq/shuttle/issues/786)) - ([f8112cb](https://github.com/shuttle-hq/shuttle/commit/f8112cb2852d9f2d0d43cc06cd20a49359c84a4b))
- Remove cargo from cargo shuttle ([#765](https://github.com/shuttle-hq/shuttle/issues/765)) - ([1d17875](https://github.com/shuttle-hq/shuttle/commit/1d17875ffc389f0df1ac566cd4943b8a7526ca40))
- Docs/add oss tenets ([#782](https://github.com/shuttle-hq/shuttle/issues/782)) - ([b802de8](https://github.com/shuttle-hq/shuttle/commit/b802de8c7eb9cb606c12f09221314dfc322761f0))
- Reference container images by full name ([#723](https://github.com/shuttle-hq/shuttle/issues/723)) - ([ba7ab11](https://github.com/shuttle-hq/shuttle/commit/ba7ab11cd5b7812cb999985985f47354abc29359))
- Fix gateway clippy ([#761](https://github.com/shuttle-hq/shuttle/issues/761)) - ([936c871](https://github.com/shuttle-hq/shuttle/commit/936c87172db5aa568cbfefd2871551af025816d2))
- Serve certificate as default ([#760](https://github.com/shuttle-hq/shuttle/issues/760)) - ([2307d96](https://github.com/shuttle-hq/shuttle/commit/2307d96195ce67c81d647baea30d08efa7987360))
- Blocked channel on gateway worker ([#758](https://github.com/shuttle-hq/shuttle/issues/758)) - ([aa513c6](https://github.com/shuttle-hq/shuttle/commit/aa513c6b9a21108305d7c64f1f96ec1dacf99c5e))
- Expect on refreshing projects ([#757](https://github.com/shuttle-hq/shuttle/issues/757)) - ([b6edc58](https://github.com/shuttle-hq/shuttle/commit/b6edc588c0653368d8f8ca70528d63dbed30eb09))

## [0.13.0](https://github.com/shuttle-hq/shuttle/compare/v0.12.0..v0.13.0) - 2023-03-27

### Features

- Polish CLI after 0.13 updates ([#750](https://github.com/shuttle-hq/shuttle/issues/750)) - ([92e3be1](https://github.com/shuttle-hq/shuttle/commit/92e3be10f9a3a0c1bca4809cebf0d163d80626af))
- Record resources in codegen ([#746](https://github.com/shuttle-hq/shuttle/issues/746)) - ([9725e00](https://github.com/shuttle-hq/shuttle/commit/9725e0073c36da4548723f72566244d4e08ab7ee))
- Resources endpoint ([#740](https://github.com/shuttle-hq/shuttle/issues/740)) - ([abd17fe](https://github.com/shuttle-hq/shuttle/commit/abd17fe380b63d0565d2f94738c91d7c2ad8135c))
- Admin command to destroy projects ([#729](https://github.com/shuttle-hq/shuttle/issues/729)) - ([8ace3ff](https://github.com/shuttle-hq/shuttle/commit/8ace3ff83483f65ea66b3d36fe1ed62b1eb5f36c))

### Bug Fixes

- Unknown resource type ([#749](https://github.com/shuttle-hq/shuttle/issues/749)) - ([d8fdc2b](https://github.com/shuttle-hq/shuttle/commit/d8fdc2b859eb936aed84782a38bb7b4b884fa6e6))

### Refactor

- Switch to resource endpoint ([#748](https://github.com/shuttle-hq/shuttle/issues/748)) - ([a69dc27](https://github.com/shuttle-hq/shuttle/commit/a69dc277960fa4d99d6b09af1bd2edad709c3fda))

### Documentation

- Update contributing project structure ([#745](https://github.com/shuttle-hq/shuttle/issues/745)) - ([97dd1a7](https://github.com/shuttle-hq/shuttle/commit/97dd1a741638fc918576abb46603875487de4e28))

### Miscellaneous Tasks

- V0.13.0 ([#755](https://github.com/shuttle-hq/shuttle/issues/755)) - ([99accab](https://github.com/shuttle-hq/shuttle/commit/99accab2bd22527b04cc2060f5f651adf71c01b7))
- Remove patch unused occurrences ([#742](https://github.com/shuttle-hq/shuttle/issues/742)) - ([a31db06](https://github.com/shuttle-hq/shuttle/commit/a31db06ed0f6e418833cf3e78bb3e967daf853f7))
- Update workspace dependencies ([#736](https://github.com/shuttle-hq/shuttle/issues/736)) - ([891f35e](https://github.com/shuttle-hq/shuttle/commit/891f35e80a2c34cc0921a477ec1c783230bbd6b9))
- Renew LetsEncrypt certificates ([#641](https://github.com/shuttle-hq/shuttle/issues/641)) - ([6843874](https://github.com/shuttle-hq/shuttle/commit/68438742f0f2d43bc980a72acd2143794ce1e33f))
- Bump rust and cargo to 1.68 ([#738](https://github.com/shuttle-hq/shuttle/issues/738)) - ([2859205](https://github.com/shuttle-hq/shuttle/commit/2859205b94d6279ba537117e29aac96f8b4dd774))
- Protoc install ([#731](https://github.com/shuttle-hq/shuttle/issues/731)) - ([5df37f4](https://github.com/shuttle-hq/shuttle/commit/5df37f466d7a50e705a5466d8c72fd7d234793f5))

### Miscellaneous

- Update resources with resourcebuilder changes ([#747](https://github.com/shuttle-hq/shuttle/issues/747)) - ([c036c6a](https://github.com/shuttle-hq/shuttle/commit/c036c6a707af7c70f9fa996cdce637a81061d8fe))
- Add feature suggestion issue template ([#737](https://github.com/shuttle-hq/shuttle/issues/737)) - ([9a45de2](https://github.com/shuttle-hq/shuttle/commit/9a45de2bb9f1add86bc29806f3ee8cfb52ec1d1d))
- Update README.md ([#698](https://github.com/shuttle-hq/shuttle/issues/698)) - ([89893cd](https://github.com/shuttle-hq/shuttle/commit/89893cd0534630644ce2e849bf59ed237e0504b2))

## [0.12.0](https://github.com/shuttle-hq/shuttle/compare/v0.11.3..v0.12.0) - 2023-03-20

### Features

- *(local)* Don't install next-runtime from git ([#718](https://github.com/shuttle-hq/shuttle/issues/718)) - ([5ea3159](https://github.com/shuttle-hq/shuttle/commit/5ea315976787e90e9e1b3948fb4016399271ba3e))
- Embed protoc in shuttle-proto ([#715](https://github.com/shuttle-hq/shuttle/issues/715)) - ([a588e25](https://github.com/shuttle-hq/shuttle/commit/a588e2564674d3bc9c2df20bc1b843577d458c53))
- Emit error when shuttle::main is named main ([#707](https://github.com/shuttle-hq/shuttle/issues/707)) - ([9f80ee8](https://github.com/shuttle-hq/shuttle/commit/9f80ee873b050260d4ef73c42c6c19830242eeea))
- Extract service integrations into separate crates ([#702](https://github.com/shuttle-hq/shuttle/issues/702)) - ([c6061be](https://github.com/shuttle-hq/shuttle/commit/c6061bede5aa7f78efaad9008be6ef71c31443a8))

### Bug Fixes

- Windows local run path bug ([#721](https://github.com/shuttle-hq/shuttle/issues/721)) - ([2d7b126](https://github.com/shuttle-hq/shuttle/commit/2d7b126681c603db296b91da344da00386e2a840))
- Static folder local run clearing file contents, add missing tests in cargo-shuttle ([#717](https://github.com/shuttle-hq/shuttle/issues/717)) - ([3cff60d](https://github.com/shuttle-hq/shuttle/commit/3cff60d67f4672b5eb9b9db01920238c26fb19c2))
- Codegen clippy ([#709](https://github.com/shuttle-hq/shuttle/issues/709)) - ([db09323](https://github.com/shuttle-hq/shuttle/commit/db09323424ea516b242d66d2279d900162bbf128))
- Respect `Cargo.lock` when building containers ([#700](https://github.com/shuttle-hq/shuttle/issues/700)) - ([9f7a482](https://github.com/shuttle-hq/shuttle/commit/9f7a48287ea452c578e98d72d4267c62a5208807))

### Refactor

- Pull out vendored protoc from shuttle-proto ([#726](https://github.com/shuttle-hq/shuttle/issues/726)) - ([5e1e527](https://github.com/shuttle-hq/shuttle/commit/5e1e527954c30b4f8bf4042d60c9c83df1e8f385))
- Move next to services ([#714](https://github.com/shuttle-hq/shuttle/issues/714)) - ([95fe7ad](https://github.com/shuttle-hq/shuttle/commit/95fe7ad29275b82b04483a8e0331288ffcb795b6))
- Rename legacy runtime to alpha ([#713](https://github.com/shuttle-hq/shuttle/issues/713)) - ([ff2ba8a](https://github.com/shuttle-hq/shuttle/commit/ff2ba8a6db4ea9488c1ad64e083acbd5684b0b5f))

### Miscellaneous Tasks

- V0.12.0 ([#727](https://github.com/shuttle-hq/shuttle/issues/727)) - ([72ce9b4](https://github.com/shuttle-hq/shuttle/commit/72ce9b4a6031b4fb5e21b030fb6673294ff64c73))
- [next] refactor: tracing ([#719](https://github.com/shuttle-hq/shuttle/issues/719)) - ([9471ed5](https://github.com/shuttle-hq/shuttle/commit/9471ed50e27f5434fc4c11e0c454679dc9736ad6))
- [next] refactor: remove ids from runtime ([#712](https://github.com/shuttle-hq/shuttle/issues/712)) - ([2ea253a](https://github.com/shuttle-hq/shuttle/commit/2ea253ae66a39537b55bebf1c4c3ccb82b2d11c6))
- [next] tests: CI go green ([#704](https://github.com/shuttle-hq/shuttle/issues/704)) - ([69819c9](https://github.com/shuttle-hq/shuttle/commit/69819c917b6b7e5278b35fb2f15662ffa5b0a79e))
- Feature/eng 486 update deployer with runtime changes ([#696](https://github.com/shuttle-hq/shuttle/issues/696)) - ([66ba530](https://github.com/shuttle-hq/shuttle/commit/66ba53071a9120b8c879627287252c4c7c227206))

### Miscellaneous

- Merge pull request #579 from shuttle-hq/shuttle-next - ([b6d7b6f](https://github.com/shuttle-hq/shuttle/commit/b6d7b6f7e8f606299d1c576e1cf5fd0ba77a6d6e))
- Next runtime not sending stop signal ([#728](https://github.com/shuttle-hq/shuttle/issues/728)) - ([ba66b33](https://github.com/shuttle-hq/shuttle/commit/ba66b339c89e9bad34363996cc3e4903d6fdddf7))
- [next] bug: misc fixes ([#725](https://github.com/shuttle-hq/shuttle/issues/725)) - ([ee04376](https://github.com/shuttle-hq/shuttle/commit/ee0437618a114ee6a59f976bac14d733b046fdbe))
- [next] bug: communicating resources ([#716](https://github.com/shuttle-hq/shuttle/issues/716)) - ([6c02135](https://github.com/shuttle-hq/shuttle/commit/6c021353c29144f23e9e13ac6bdf26acce24617a))
- Eng 497 update or remove the docs in shuttle ([#710](https://github.com/shuttle-hq/shuttle/issues/710)) - ([f21d0dd](https://github.com/shuttle-hq/shuttle/commit/f21d0dde37eeaf02f660b832964edac94aef0160))
- [next] refactor: update runtime manager ([#711](https://github.com/shuttle-hq/shuttle/issues/711)) - ([add6a8e](https://github.com/shuttle-hq/shuttle/commit/add6a8e8e3b4df04697bf5e02517de5beb43dfc6))
- V0.12.0-rc1 ([#708](https://github.com/shuttle-hq/shuttle/issues/708)) - ([9f73d61](https://github.com/shuttle-hq/shuttle/commit/9f73d61f330b0f66ce926081581703f115629d59))
- Eng 484 update init with codegen changes ([#706](https://github.com/shuttle-hq/shuttle/issues/706)) - ([918829b](https://github.com/shuttle-hq/shuttle/commit/918829bde43575e16bd729b3232794b4c78906e5))
- Merge remote-tracking branch 'upstream/main' into shuttle-next - ([4e88558](https://github.com/shuttle-hq/shuttle/commit/4e88558df14be4bda47a356119206a2a30653150))

## [0.11.3](https://github.com/shuttle-hq/shuttle/compare/v0.11.2..v0.11.3) - 2023-03-08

### Documentation

- Add note about git tags in contrib ([#691](https://github.com/shuttle-hq/shuttle/issues/691)) - ([67cf6bd](https://github.com/shuttle-hq/shuttle/commit/67cf6bd91d9aa0df0a4e2b5a8752f71510fb5f5e))

### Miscellaneous Tasks

- *(typos)* Fix typos ([#682](https://github.com/shuttle-hq/shuttle/issues/682)) - ([a6279c6](https://github.com/shuttle-hq/shuttle/commit/a6279c69b873c085c017abb3b7f0798ac8d06d19))
- V0.11.3 ([#695](https://github.com/shuttle-hq/shuttle/issues/695)) - ([349f578](https://github.com/shuttle-hq/shuttle/commit/349f57859b4805a18f2b99f12612e9787eae44f0))

### Miscellaneous

- Idle static folder ([#692](https://github.com/shuttle-hq/shuttle/issues/692)) - ([9fa862e](https://github.com/shuttle-hq/shuttle/commit/9fa862ea1be5b3298d351c25c53f4c0c591fedbc))
- Fix indentation for poise init example ([#687](https://github.com/shuttle-hq/shuttle/issues/687)) - ([0fecace](https://github.com/shuttle-hq/shuttle/commit/0fecace65ba38af1c67ac6f29051220cc6184661))
- Fix/move docker profiles to dev ([#674](https://github.com/shuttle-hq/shuttle/issues/674)) - ([91f12b4](https://github.com/shuttle-hq/shuttle/commit/91f12b4e138c48b392db36c456809db2fe00c87e))

## [0.11.2](https://github.com/shuttle-hq/shuttle/compare/v0.11.1..v0.11.2) - 2023-03-02

### Features

- Stop idle deployers ([#627](https://github.com/shuttle-hq/shuttle/issues/627)) - ([1d1c451](https://github.com/shuttle-hq/shuttle/commit/1d1c4512797780e719c5ebe1178a56f231bcde7c))
- Add bug report and PR templates ([#661](https://github.com/shuttle-hq/shuttle/issues/661)) - ([bc2d0eb](https://github.com/shuttle-hq/shuttle/commit/bc2d0ebabd7a4514c62098d843dc83949a047dc3))

### Documentation

- Fix and update all markdown files + some more ([#666](https://github.com/shuttle-hq/shuttle/issues/666)) - ([da0fd4e](https://github.com/shuttle-hq/shuttle/commit/da0fd4e97745f415826770c2246d08b5960c4f25))

### Miscellaneous Tasks

- V0.11.2 ([#672](https://github.com/shuttle-hq/shuttle/issues/672)) - ([262ef1a](https://github.com/shuttle-hq/shuttle/commit/262ef1ab00bf29e7daf56e34a72f61b8138f74ba))
- Opt out of panamax ([#576](https://github.com/shuttle-hq/shuttle/issues/576)) - ([631ee96](https://github.com/shuttle-hq/shuttle/commit/631ee96e9067a5846b64035dea9efa51a37c64c7))

### Miscellaneous

- Ignore misc lockfiles ([#670](https://github.com/shuttle-hq/shuttle/issues/670)) - ([2e4f904](https://github.com/shuttle-hq/shuttle/commit/2e4f904a8e050e44c9ae0606396445e6fcedf356))
- Update chrono and mongodb ([#664](https://github.com/shuttle-hq/shuttle/issues/664)) - ([93d2b8d](https://github.com/shuttle-hq/shuttle/commit/93d2b8d39787e31a7a83e091d72d89a35ce418bd))

## [0.11.1](https://github.com/shuttle-hq/shuttle/compare/v0.11.0..v0.11.1) - 2023-03-02

### Features

- Unbox InjectPropagation and ExtractPropagation ([#663](https://github.com/shuttle-hq/shuttle/issues/663)) - ([77fb6cd](https://github.com/shuttle-hq/shuttle/commit/77fb6cd0c1d0fcbe12caf423861316f39ca11dd2))
- Bump panamax, remove docker stats receiver ([#660](https://github.com/shuttle-hq/shuttle/issues/660)) - ([a001bda](https://github.com/shuttle-hq/shuttle/commit/a001bda0a2786e0e877bbf93e1506770a102d13c))

### Refactor

- Unboxing AdminSecret ([#662](https://github.com/shuttle-hq/shuttle/issues/662)) - ([78dab59](https://github.com/shuttle-hq/shuttle/commit/78dab59297a5007e56d35c8b0913bf6190de855e))

### Miscellaneous Tasks

- V0.11.1 ([#669](https://github.com/shuttle-hq/shuttle/issues/669)) - ([283eb0b](https://github.com/shuttle-hq/shuttle/commit/283eb0b70135323ec03f4dcd327323dfe8fdf465))

### Miscellaneous

- Ws not getting logs from broadcast channel ([#667](https://github.com/shuttle-hq/shuttle/issues/667)) - ([e8fdad3](https://github.com/shuttle-hq/shuttle/commit/e8fdad3efe345723248e18e663d24762c66fe7a4))

## [0.11.0](https://github.com/shuttle-hq/shuttle/compare/v0.10.0..v0.11.0) - 2023-02-27

### Features

- Auth cache ([#643](https://github.com/shuttle-hq/shuttle/issues/643)) - ([6686657](https://github.com/shuttle-hq/shuttle/commit/6686657df487c7222b981fdcf953fb0ca2270d73))
- Allow admin scoped user to recreate any project ([#651](https://github.com/shuttle-hq/shuttle/issues/651)) - ([5187f6a](https://github.com/shuttle-hq/shuttle/commit/5187f6a53b47c9c96bf3a462b02864ca15ace936))
- OpenTelemetry collector ([#649](https://github.com/shuttle-hq/shuttle/issues/649)) - ([f8d5ac8](https://github.com/shuttle-hq/shuttle/commit/f8d5ac8fb670e2f499bfb5855d4a37eafdcd1628))
- Allow filtering projects by project status ([#613](https://github.com/shuttle-hq/shuttle/issues/613)) - ([94a0708](https://github.com/shuttle-hq/shuttle/commit/94a070832e9953773eec7fe5330583ba0adcc95a))
- Implement rpc method in provisioner that allows for the deletion of resources ([#622](https://github.com/shuttle-hq/shuttle/issues/622)) - ([28a8abe](https://github.com/shuttle-hq/shuttle/commit/28a8abe9df027633bfa1732d52b95dda3f1d7e70))
- Convert api key to jwt ([#640](https://github.com/shuttle-hq/shuttle/issues/640)) - ([a89136a](https://github.com/shuttle-hq/shuttle/commit/a89136a8fa7694421a2c92c3f5e03bc8903416b6))
- Session manager ([#638](https://github.com/shuttle-hq/shuttle/issues/638)) - ([e8ab443](https://github.com/shuttle-hq/shuttle/commit/e8ab443730f63089de6a8875ebbf037d39f7d010))
- Add a users layer ([#633](https://github.com/shuttle-hq/shuttle/issues/633)) - ([0865c3b](https://github.com/shuttle-hq/shuttle/commit/0865c3b4012297631e992fdecdd1a1f7b88dc469))
- Create an auth project + clap ([#630](https://github.com/shuttle-hq/shuttle/issues/630)) - ([b3e11b5](https://github.com/shuttle-hq/shuttle/commit/b3e11b5ec0eb129dfbe06f90b99c785ea5334126))

### Refactor

- Update opentelemetry in all our crates ([#652](https://github.com/shuttle-hq/shuttle/issues/652)) - ([c7d5e56](https://github.com/shuttle-hq/shuttle/commit/c7d5e566f0da45ca799091f438aa81ea9b17ea3b))
- Get project name from label ([#646](https://github.com/shuttle-hq/shuttle/issues/646)) - ([6ee5a66](https://github.com/shuttle-hq/shuttle/commit/6ee5a6627f2388fca0c7ff178ccc511ae2e1bded))

### Documentation

- Clean up the contributing doc ([#644](https://github.com/shuttle-hq/shuttle/issues/644)) - ([bb41997](https://github.com/shuttle-hq/shuttle/commit/bb41997e4da4abe259068ce7912a0258394aa7e8))

### Miscellaneous Tasks

- V0.11.0 ([#654](https://github.com/shuttle-hq/shuttle/issues/654)) - ([d7a5333](https://github.com/shuttle-hq/shuttle/commit/d7a53339825de01c7f4a0f2c426fd5a967d6ec4b))
- Update examples submodule ([#656](https://github.com/shuttle-hq/shuttle/issues/656)) - ([df7cb49](https://github.com/shuttle-hq/shuttle/commit/df7cb4970c88a4f382f9f19a4e5d022affc397e7))
- Cache public key ([#655](https://github.com/shuttle-hq/shuttle/issues/655)) - ([13d8bf0](https://github.com/shuttle-hq/shuttle/commit/13d8bf0a5dc48cb9c68636d4236ddbf7b54107ed))
- [auth] feat: axum with routes ([#632](https://github.com/shuttle-hq/shuttle/issues/632)) - ([b7bcbe1](https://github.com/shuttle-hq/shuttle/commit/b7bcbe19642ac4574862f217b840e02ec6243f67))

### Miscellaneous

- Unbox the ClaimService and Scoped futures ([#653](https://github.com/shuttle-hq/shuttle/issues/653)) - ([fb7c5ae](https://github.com/shuttle-hq/shuttle/commit/fb7c5ae253826137e7349b6f3222a4f26b6e6ad4))
- Remove target from bin build name ([#650](https://github.com/shuttle-hq/shuttle/issues/650)) - ([f45c6ca](https://github.com/shuttle-hq/shuttle/commit/f45c6cae2cdc072370c5c5a156a7ae9240fd50e5))
- [auth] refactor: update gateway deployer and provisioner ([#642](https://github.com/shuttle-hq/shuttle/issues/642)) - ([e8536e8](https://github.com/shuttle-hq/shuttle/commit/e8536e881e33a93491750e085615b71665da988d))
- ([#634](https://github.com/shuttle-hq/shuttle/issues/634)) restore custom domain when recreating project ([#637](https://github.com/shuttle-hq/shuttle/issues/637)) - ([9d9035f](https://github.com/shuttle-hq/shuttle/commit/9d9035f81b724ee1bbe1cb6a593d383c828d5c26))
- Fix for install docker buildx issue #29 ([#636](https://github.com/shuttle-hq/shuttle/issues/636)) - ([392021e](https://github.com/shuttle-hq/shuttle/commit/392021ecbdc6653bf2eaa9c73a994e79a1d801ee))
- [auth] feat: public key endpoint ([#639](https://github.com/shuttle-hq/shuttle/issues/639)) - ([c2264c5](https://github.com/shuttle-hq/shuttle/commit/c2264c5eb0602d094bf1286ffb5e165b3c1b3371))
- Optimize sqlite db in deployer and gateway ([#623](https://github.com/shuttle-hq/shuttle/issues/623)) - ([a9ab3e6](https://github.com/shuttle-hq/shuttle/commit/a9ab3e692547b70d3f9434db5b0d03c015c1723a))
- [auth] feat: add an auth module to shuttle common ([#635](https://github.com/shuttle-hq/shuttle/issues/635)) - ([45d2b12](https://github.com/shuttle-hq/shuttle/commit/45d2b12e399c58e6b369dc47dd66f32fcab4cb4b))
- Update README.md - ([7e4d0d7](https://github.com/shuttle-hq/shuttle/commit/7e4d0d754dff80e6f07166016d1de3f1a14dca27))
- Revive ([#631](https://github.com/shuttle-hq/shuttle/issues/631)) - ([248ae9b](https://github.com/shuttle-hq/shuttle/commit/248ae9b7185479e161fdcfe92335444bdab85eac))

## [0.10.0](https://github.com/shuttle-hq/shuttle/compare/v0.9.0..v0.10.0) - 2023-02-13

### Features

- Show progress bar on stopping service ([#628](https://github.com/shuttle-hq/shuttle/issues/628)) - ([524f5d7](https://github.com/shuttle-hq/shuttle/commit/524f5d7a0045aeb66585e8f9b362b043359e4f61))
- Add dev/prod secrets functionality ([#610](https://github.com/shuttle-hq/shuttle/issues/610)) - ([21549a9](https://github.com/shuttle-hq/shuttle/commit/21549a9d51029512897d0f550763e59b6a727946))
- Retry on bollard errors ([#620](https://github.com/shuttle-hq/shuttle/issues/620)) - ([f380e60](https://github.com/shuttle-hq/shuttle/commit/f380e6055baa14389d31371aea0f9b3cff8dce55))
- Rename delete to stop ([#619](https://github.com/shuttle-hq/shuttle/issues/619)) - ([a0b412e](https://github.com/shuttle-hq/shuttle/commit/a0b412ec0a359483421377799652042e89bcdade))
- Respect $PORT environment variable for local run ([#571](https://github.com/shuttle-hq/shuttle/issues/571)) - ([c186c99](https://github.com/shuttle-hq/shuttle/commit/c186c9997be7f9ac06a42a3361fc08e7f932fedd))
- Migrate from the tempdir crate to tempfile ([#603](https://github.com/shuttle-hq/shuttle/issues/603)) - ([ec5183b](https://github.com/shuttle-hq/shuttle/commit/ec5183b5a46f48f7bf8e0603800730dfba13ea6c))
- Add release profile flag to local run command ([#611](https://github.com/shuttle-hq/shuttle/issues/611)) - ([92ddf6e](https://github.com/shuttle-hq/shuttle/commit/92ddf6ea85933ae8ca5799a4bbe772044e59f6f7))
- Add logout function ([#595](https://github.com/shuttle-hq/shuttle/issues/595)) - ([1a81711](https://github.com/shuttle-hq/shuttle/commit/1a817117f87a6705e15c825b47185e8184cc66a7))

### Bug Fixes

- Add repo url to cargo-shuttle for binstall ([#608](https://github.com/shuttle-hq/shuttle/issues/608)) - ([8a55cce](https://github.com/shuttle-hq/shuttle/commit/8a55cceb16574f01b6fec7d3cdb2b262ff505d62))

### Miscellaneous Tasks

- Update examples submodule ([#626](https://github.com/shuttle-hq/shuttle/issues/626)) - ([7baf511](https://github.com/shuttle-hq/shuttle/commit/7baf5110202907051343e3bae64215a2ae352125))
- Bin builds tag persistence ([#605](https://github.com/shuttle-hq/shuttle/issues/605)) - ([496a860](https://github.com/shuttle-hq/shuttle/commit/496a8605e57c10e9ef97128537518578edf86044))

### Miscellaneous

- V0.10.0 ([#625](https://github.com/shuttle-hq/shuttle/issues/625)) - ([4668291](https://github.com/shuttle-hq/shuttle/commit/46682919251388e5884d58046c981ff95a8a2480))
- Make Stopped a sink state ([#624](https://github.com/shuttle-hq/shuttle/issues/624)) - ([407284a](https://github.com/shuttle-hq/shuttle/commit/407284a56564bf7c15f793b2066b65f9b64719f7))

## [0.9.0](https://github.com/shuttle-hq/shuttle/compare/v0.8.1..v0.9.0) - 2023-01-27

### Features

- Build prod images in release profile ([#590](https://github.com/shuttle-hq/shuttle/issues/590)) - ([2242cbc](https://github.com/shuttle-hq/shuttle/commit/2242cbc23a77a95c6e22ba998148b1d16fec5417))
- Allow string interpolation on resource options ([#597](https://github.com/shuttle-hq/shuttle/issues/597)) - ([083cc6a](https://github.com/shuttle-hq/shuttle/commit/083cc6a7c4f9cf6a151649bc5f7a3dbc64349972))
- Local uri ([#596](https://github.com/shuttle-hq/shuttle/issues/596)) - ([bd57774](https://github.com/shuttle-hq/shuttle/commit/bd5777464a8830ae76a1f7b65dc766f5894fbb90))
- Trim the service loader, unpin tokio ([#681](https://github.com/shuttle-hq/shuttle/issues/681)) - ([8be4742](https://github.com/shuttle-hq/shuttle/commit/8be4742b1c075834364aed9c5991c3a50dabc1a2))
- Extract next runtime into separate binary ([#679](https://github.com/shuttle-hq/shuttle/issues/679)) - ([52096fc](https://github.com/shuttle-hq/shuttle/commit/52096fc6d895ab0146f24313d112d13d2198a582))
- Remove box self from services, remove syncwrapper from axum service ([#677](https://github.com/shuttle-hq/shuttle/issues/677)) - ([69b82e6](https://github.com/shuttle-hq/shuttle/commit/69b82e605d3b168f2e803767f0847799d0447dea))
- Build our images with the release profile ([#583](https://github.com/shuttle-hq/shuttle/issues/583)) - ([d191d66](https://github.com/shuttle-hq/shuttle/commit/d191d66ac16146e4af3f401bdb917396785e2dd4))
- Add flag for router IP local run ([#565](https://github.com/shuttle-hq/shuttle/issues/565)) - ([8f71804](https://github.com/shuttle-hq/shuttle/commit/8f7180476777c6134560bcae52f585b5103e4933))
- Support Poise ([#560](https://github.com/shuttle-hq/shuttle/issues/560)) - ([0599a13](https://github.com/shuttle-hq/shuttle/commit/0599a1304a3e5cee464dadc341b340117ae6bfed))
- Create subcommand to list all projects of calling account ([#553](https://github.com/shuttle-hq/shuttle/issues/553)) - ([cb342fd](https://github.com/shuttle-hq/shuttle/commit/cb342fd980db24841fe89e14d1f601b863377ae5))

### Refactor

- Hide some pg catalogs ([#593](https://github.com/shuttle-hq/shuttle/issues/593)) - ([cde9e36](https://github.com/shuttle-hq/shuttle/commit/cde9e366d0c9d827d56ebcdf239cb9177c7c9d37))
- Find code to wrap legacy runtime ([#675](https://github.com/shuttle-hq/shuttle/issues/675)) - ([91a9fdd](https://github.com/shuttle-hq/shuttle/commit/91a9fdde5b3ba505ee2ae5642e4fa4eefce390f2))
- Unwraps and mor ([#587](https://github.com/shuttle-hq/shuttle/issues/587)) - ([a8b6166](https://github.com/shuttle-hq/shuttle/commit/a8b616654f87427b5bb639901282b01f3b420cf4))
- Proto fixups ([#585](https://github.com/shuttle-hq/shuttle/issues/585)) - ([325b90f](https://github.com/shuttle-hq/shuttle/commit/325b90f966d1f01122cc04d94df94f0266bd758f))

### Testing

- Fixes ([#554](https://github.com/shuttle-hq/shuttle/issues/554)) - ([b00b5e8](https://github.com/shuttle-hq/shuttle/commit/b00b5e88a49b2425f701c94ca46b2e39366511c9))

### Miscellaneous Tasks

- V0.9.0 ([#600](https://github.com/shuttle-hq/shuttle/issues/600)) - ([79ff57e](https://github.com/shuttle-hq/shuttle/commit/79ff57e1a2e712748803d20e7aa51b1f9dbf72af))
- Update examples submodule ([#601](https://github.com/shuttle-hq/shuttle/issues/601)) - ([a038cd6](https://github.com/shuttle-hq/shuttle/commit/a038cd6e5240862348101b1521884f7bf8baef8c))
- Resolve CI errors in shuttle-next ([#580](https://github.com/shuttle-hq/shuttle/issues/580)) - ([adf8926](https://github.com/shuttle-hq/shuttle/commit/adf89267e44874022fc20ab7f8038d131d7578b0))
- Eng 465 update all the codegens ([#686](https://github.com/shuttle-hq/shuttle/issues/686)) - ([3699f7f](https://github.com/shuttle-hq/shuttle/commit/3699f7f69e3f48d86f9c7b98b6af66c4b32a65c8))
- Feature/eng 477 make wasm dependencies optional ([#688](https://github.com/shuttle-hq/shuttle/issues/688)) - ([a93ba51](https://github.com/shuttle-hq/shuttle/commit/a93ba5111700c1067a0a0a357c5ca490a5ad24a8))
- Cleanup fds and tmp ([#586](https://github.com/shuttle-hq/shuttle/issues/586)) - ([35c0660](https://github.com/shuttle-hq/shuttle/commit/35c06603d77265138e2f02b4420f740bbb2dfea0))
- Fix bin builds ([#546](https://github.com/shuttle-hq/shuttle/issues/546)) - ([d113ca1](https://github.com/shuttle-hq/shuttle/commit/d113ca1c83646e2be0a9a60bd839a22a40964bc1))
- Dependencies compiled with incompatible versions of rustc ([#545](https://github.com/shuttle-hq/shuttle/issues/545)) - ([45eadce](https://github.com/shuttle-hq/shuttle/commit/45eadcefe65a1c92694adf3c3313bcfd8dfef5ca))
- Add Makefile command for Windows to convert .sh files to LF format ([#555](https://github.com/shuttle-hq/shuttle/issues/555)) - ([2d0f338](https://github.com/shuttle-hq/shuttle/commit/2d0f33851f7d9b32e67719c5a3a86039fdd54a9f))

### Miscellaneous

- Added feedback ([#592](https://github.com/shuttle-hq/shuttle/issues/592)) - ([432ffb4](https://github.com/shuttle-hq/shuttle/commit/432ffb467d1e4cfb0036b05cb71b09704b0eb6ac))
- Upgraded to clap v4 ([#570](https://github.com/shuttle-hq/shuttle/issues/570)) - ([c68c04e](https://github.com/shuttle-hq/shuttle/commit/c68c04e3146defae959d757d5e48f7daab63d83a))
- Merge remote-tracking branch 'upstream/main' into shuttle-next - ([60be73d](https://github.com/shuttle-hq/shuttle/commit/60be73da5509d8114e983032b86fba83639f31ac))
- Eng 483 trim and fix the tests in shuttle-service ([#693](https://github.com/shuttle-hq/shuttle/issues/693)) - ([4e1690d](https://github.com/shuttle-hq/shuttle/commit/4e1690d8cc5da7986513014d7bbd382303e741a4))
- Remove tokio runtime from all resources ([#680](https://github.com/shuttle-hq/shuttle/issues/680)) - ([3489e83](https://github.com/shuttle-hq/shuttle/commit/3489e83cfabd775fc6c54dbba0339c642b69be85))
- Feature/eng 378 axum wasm multiple handlers per endpoint ([#588](https://github.com/shuttle-hq/shuttle/issues/588)) - ([3dc3ac7](https://github.com/shuttle-hq/shuttle/commit/3dc3ac76d3fc06728416eedd932fff0da4218230))
- Remove unneeded codegen feature ([#584](https://github.com/shuttle-hq/shuttle/issues/584)) - ([05f7469](https://github.com/shuttle-hq/shuttle/commit/05f746967c5a556fef8c7f8259b0fd62f8849410))
- Merge remote-tracking branch 'upstream/main' into shuttle-next - ([8414714](https://github.com/shuttle-hq/shuttle/commit/841471405b815e0480796eb650f7ada282c4d67b))
- No networks ([#541](https://github.com/shuttle-hq/shuttle/issues/541)) - ([604271a](https://github.com/shuttle-hq/shuttle/commit/604271a804129cf02bdbfc7144a4a3b672c0984b))
- Update contributing ([#556](https://github.com/shuttle-hq/shuttle/issues/556)) - ([85268c9](https://github.com/shuttle-hq/shuttle/commit/85268c9dd5526123feb501003134e6f683c32327))
- Deployer drifting state ([#548](https://github.com/shuttle-hq/shuttle/issues/548)) - ([eda4769](https://github.com/shuttle-hq/shuttle/commit/eda476953884a28efd53d8d56563fc5189edafcb))
- Remove deprecated auth command ([#550](https://github.com/shuttle-hq/shuttle/issues/550)) - ([b1dbdb7](https://github.com/shuttle-hq/shuttle/commit/b1dbdb7253c4a4a7369cf012a78dcead58d7fe76))

## [0.8.1](https://github.com/shuttle-hq/shuttle/compare/v0.7.2..v0.8.1) - 2022-12-14

### Features

- *(gateway)* Add custom domains table and routing ([#465](https://github.com/shuttle-hq/shuttle/issues/465)) - ([3ab6c71](https://github.com/shuttle-hq/shuttle/commit/3ab6c71afcf781c75847c7b3d74e3beca1a6f24d))
- *(gateway,deployer)* Add more tracing events ([#500](https://github.com/shuttle-hq/shuttle/issues/500)) - ([8387138](https://github.com/shuttle-hq/shuttle/commit/83871382f39b0139fb5d0d44fc0fd0b5ed5213d4))
- Add cron job for syncing mirror ([#537](https://github.com/shuttle-hq/shuttle/issues/537)) - ([0afa129](https://github.com/shuttle-hq/shuttle/commit/0afa1296db7e751d1f32dce96310632fdcf08b2b))
- Temp validation of project name in gateway ([#534](https://github.com/shuttle-hq/shuttle/issues/534)) - ([a7e7ed6](https://github.com/shuttle-hq/shuttle/commit/a7e7ed61e183aaa8291295ec596663150d91b5a9))
- Per-project parallelism ([#533](https://github.com/shuttle-hq/shuttle/issues/533)) - ([ae8ee01](https://github.com/shuttle-hq/shuttle/commit/ae8ee0143612c2485924ed109ef7742a90d982ce))
- Build queue ([#532](https://github.com/shuttle-hq/shuttle/issues/532)) - ([5e604b4](https://github.com/shuttle-hq/shuttle/commit/5e604b4847d27a1d50a30bc858aaee11ccf05384))
- Add panamax for mirroring crates.io ([#525](https://github.com/shuttle-hq/shuttle/issues/525)) - ([d60f642](https://github.com/shuttle-hq/shuttle/commit/d60f642184c90aeab7c2216754bc0cd3d06a7d26))
- 'clean' subcommand ([#530](https://github.com/shuttle-hq/shuttle/issues/530)) - ([8e93e87](https://github.com/shuttle-hq/shuttle/commit/8e93e8718777967b2bc5ac3b6f3029108e1df092))
- Canonicalize before trace ([#531](https://github.com/shuttle-hq/shuttle/issues/531)) - ([18767f0](https://github.com/shuttle-hq/shuttle/commit/18767f07f3ca4f05256e350cdbbdbcda25bbccf4))
- Create `init` project from correct dir ([#518](https://github.com/shuttle-hq/shuttle/issues/518)) - ([519ac04](https://github.com/shuttle-hq/shuttle/commit/519ac04ee143bd56162fd7a47fe47b3f6ca15a1f))
- Build tests in release profile, limit build workers ([#514](https://github.com/shuttle-hq/shuttle/issues/514)) - ([a37903a](https://github.com/shuttle-hq/shuttle/commit/a37903a51d6df28b2240c9d87221449a028d653d))
- Find (soon to be) invalid project names ([#479](https://github.com/shuttle-hq/shuttle/issues/479)) - ([2e6ac41](https://github.com/shuttle-hq/shuttle/commit/2e6ac41400e335cff4ea86319ec654d9c108b30a))
- Add spinner wait for `project new` and `project status --follow` ([#503](https://github.com/shuttle-hq/shuttle/issues/503)) - ([b597eef](https://github.com/shuttle-hq/shuttle/commit/b597eefca28399f994f455988aa97b6cf0b66296))
- Implement workspace inheritance ([#506](https://github.com/shuttle-hq/shuttle/issues/506)) - ([8052e87](https://github.com/shuttle-hq/shuttle/commit/8052e878e6dea9400dc17176e4ee01414812eeeb))
- Make the folder configurable ([#508](https://github.com/shuttle-hq/shuttle/issues/508)) - ([3d5c55b](https://github.com/shuttle-hq/shuttle/commit/3d5c55b513740efc6f48cbc297749f90e7618cc6))
- Bump pinned rust version to 1.65 ([#504](https://github.com/shuttle-hq/shuttle/issues/504)) - ([ca97f03](https://github.com/shuttle-hq/shuttle/commit/ca97f03b596363b7bc4a57339a99bb6576ff36aa))
- Interactive project initialization ([#498](https://github.com/shuttle-hq/shuttle/issues/498)) - ([887dce4](https://github.com/shuttle-hq/shuttle/commit/887dce49dd402d608c344225ddcf3f11b96d88aa))
- Bump rust to 1.64, bump dependencies ([#495](https://github.com/shuttle-hq/shuttle/issues/495)) - ([961964a](https://github.com/shuttle-hq/shuttle/commit/961964a8df9d2561e0d524c8731922ab082ba526))
- Static file support for a single folder ([#501](https://github.com/shuttle-hq/shuttle/issues/501)) - ([6c3025b](https://github.com/shuttle-hq/shuttle/commit/6c3025be742b5cfbfe9127f66d42f88512152c34))
- Gateway restores removed containers ([#485](https://github.com/shuttle-hq/shuttle/issues/485)) - ([b748493](https://github.com/shuttle-hq/shuttle/commit/b748493fae7b0c56b1ffc88bd2a099900bd3077a))
- TLS acceptor with SNI resolver ([#471](https://github.com/shuttle-hq/shuttle/issues/471)) - ([3bd6f0f](https://github.com/shuttle-hq/shuttle/commit/3bd6f0fb1aa4f4f2312d2e703c53b61213487789))
- Add a custom domains admin route ([#473](https://github.com/shuttle-hq/shuttle/issues/473)) - ([7b80c45](https://github.com/shuttle-hq/shuttle/commit/7b80c45b227f7f6355e824438aee33c4a444c858))
- Verify project exists before sending destroy task ([#474](https://github.com/shuttle-hq/shuttle/issues/474)) - ([e10f096](https://github.com/shuttle-hq/shuttle/commit/e10f09614659efdf20ed518ae852a1e185207a06))
- Make deployer only answer its own project ([#466](https://github.com/shuttle-hq/shuttle/issues/466)) - ([001dbcf](https://github.com/shuttle-hq/shuttle/commit/001dbcfcf47066713598f00a98f4972de450ff3b))
- Create a new admin cli binary crate ([#462](https://github.com/shuttle-hq/shuttle/issues/462)) - ([7471c08](https://github.com/shuttle-hq/shuttle/commit/7471c08b3926e680fac8464dea543121c7153d14))
- Prefetch shuttle-service crates ([#461](https://github.com/shuttle-hq/shuttle/issues/461)) - ([5fbf7c9](https://github.com/shuttle-hq/shuttle/commit/5fbf7c9ab37f14d951b8e841f0ba3959d136346c))
- Add account_tier column ([#458](https://github.com/shuttle-hq/shuttle/issues/458)) - ([b1eee6d](https://github.com/shuttle-hq/shuttle/commit/b1eee6df7e7e7302a50e737b8e926b933bfdc370))

### Bug Fixes

- *(deployer)* Keep Cargo.lock between deployments ([#517](https://github.com/shuttle-hq/shuttle/issues/517)) - ([24657bc](https://github.com/shuttle-hq/shuttle/commit/24657bc2e5fc8e1a1de9744b239b9ddf4938ed8c))
- Actix integration with state ([#523](https://github.com/shuttle-hq/shuttle/issues/523)) - ([489b925](https://github.com/shuttle-hq/shuttle/commit/489b92595f0e41290d376a8965678c0263a2ef85))
- Make nice ([#512](https://github.com/shuttle-hq/shuttle/issues/512)) - ([6bbda80](https://github.com/shuttle-hq/shuttle/commit/6bbda8051d07b6f634410f0932cc2ee49ff36c84))
- Capitalise correctly ([#511](https://github.com/shuttle-hq/shuttle/issues/511)) - ([0c4eb94](https://github.com/shuttle-hq/shuttle/commit/0c4eb943c27afa136ebd40446b95c01aeac23561))
- Backend bumps and hot fixes ([#487](https://github.com/shuttle-hq/shuttle/issues/487)) - ([e3fb067](https://github.com/shuttle-hq/shuttle/commit/e3fb067b3e62b399b800f591df5404797f7581ce))
- Custom domain routing ([#484](https://github.com/shuttle-hq/shuttle/issues/484)) - ([d8fedbd](https://github.com/shuttle-hq/shuttle/commit/d8fedbd143806f316143ebb28c33927524b8d174))
- Add timeout to health checks ([#468](https://github.com/shuttle-hq/shuttle/issues/468)) - ([b4055af](https://github.com/shuttle-hq/shuttle/commit/b4055af55e3991ca39db9d3bfae3efa7d9e953db))
- Broken link ([#467](https://github.com/shuttle-hq/shuttle/issues/467)) - ([b6bd64c](https://github.com/shuttle-hq/shuttle/commit/b6bd64ca4cdb6417751c5487b0f0b327edec0247))
- Gateway state drifts, health checks and project recreation ([#447](https://github.com/shuttle-hq/shuttle/issues/447)) - ([9d5e345](https://github.com/shuttle-hq/shuttle/commit/9d5e345f0d1fcc03557bb6784a43114d14769adc))
- Wrap around common::ProjectName for parsing ([#451](https://github.com/shuttle-hq/shuttle/issues/451)) - ([bd0c381](https://github.com/shuttle-hq/shuttle/commit/bd0c381c79c6144d828bc7856d08b4c0fe085acf))

### Refactor

- Remove prefetch ([#539](https://github.com/shuttle-hq/shuttle/issues/539)) - ([84dd5fa](https://github.com/shuttle-hq/shuttle/commit/84dd5fad4b3b7bdd118351bdfeb73b95e2290a63))
- Release build slot parse type correctly ([#538](https://github.com/shuttle-hq/shuttle/issues/538)) - ([5d638dc](https://github.com/shuttle-hq/shuttle/commit/5d638dca3da20be34ec861c6bce520e3190e9141))
- Don't crash when failing to release slot ([#536](https://github.com/shuttle-hq/shuttle/issues/536)) - ([5ed12ad](https://github.com/shuttle-hq/shuttle/commit/5ed12adc45867a6831a4278c59f628e054d0528b))
- Missed axum 0.6 update ([#513](https://github.com/shuttle-hq/shuttle/issues/513)) - ([7525c7a](https://github.com/shuttle-hq/shuttle/commit/7525c7af760fc28eb94e1d13e8f434372417c2cb))
- Switch away from cargo package ([#507](https://github.com/shuttle-hq/shuttle/issues/507)) - ([d9d6d3e](https://github.com/shuttle-hq/shuttle/commit/d9d6d3e4923206059ac1fb4f4214b487356448e2))
- More metrics ([#475](https://github.com/shuttle-hq/shuttle/issues/475)) - ([9a85dc4](https://github.com/shuttle-hq/shuttle/commit/9a85dc44daa3529f8d51c88ac44b118183301e4b))
- Base client error off response status code ([#470](https://github.com/shuttle-hq/shuttle/issues/470)) - ([3bcc683](https://github.com/shuttle-hq/shuttle/commit/3bcc683b408fee8f8d59a513651d0fd9b6165509))
- Tf files have been moved to shuttle-hq/terraform-aws-shuttle - ([6c848bf](https://github.com/shuttle-hq/shuttle/commit/6c848bf351cd52650a40d5314869127ff9f2e948))

### Miscellaneous Tasks

- Bump cargo-shuttle to 0.8.1 ([#540](https://github.com/shuttle-hq/shuttle/issues/540)) - ([998fff7](https://github.com/shuttle-hq/shuttle/commit/998fff7ee056e35b5be000c50f96971f82f07b78))
- Remove build and push req on build binaries ([#535](https://github.com/shuttle-hq/shuttle/issues/535)) - ([027b50d](https://github.com/shuttle-hq/shuttle/commit/027b50df69c1c03a3059f64e80fc692df9d2d893))
- Bump examples ([#522](https://github.com/shuttle-hq/shuttle/issues/522)) - ([5b9769e](https://github.com/shuttle-hq/shuttle/commit/5b9769e64a5cbe6d0350ae4b7e1cbb98f5e13988))
- 0.8.0 ([#521](https://github.com/shuttle-hq/shuttle/issues/521)) - ([5c19ea3](https://github.com/shuttle-hq/shuttle/commit/5c19ea38855c525d4a7e73d06303eb572045b950))
- Env updates ([#509](https://github.com/shuttle-hq/shuttle/issues/509)) - ([650e3f5](https://github.com/shuttle-hq/shuttle/commit/650e3f5cc2390dd97b894c8ac778cdf9da842c2a))
- Build binary ([#483](https://github.com/shuttle-hq/shuttle/issues/483)) - ([6a551d1](https://github.com/shuttle-hq/shuttle/commit/6a551d1f7afe75c177b26b4371eb17b090c34ef9))
- Feature/support actix web ([#491](https://github.com/shuttle-hq/shuttle/issues/491)) - ([57ec829](https://github.com/shuttle-hq/shuttle/commit/57ec82950c2dfde07f2474063d69f85c74edec42))
- Green ([#482](https://github.com/shuttle-hq/shuttle/issues/482)) - ([74aeb46](https://github.com/shuttle-hq/shuttle/commit/74aeb46cdf184916c651b469449d1da77277022c))
- Restructure repo ([#453](https://github.com/shuttle-hq/shuttle/issues/453)) - ([8a6efb8](https://github.com/shuttle-hq/shuttle/commit/8a6efb8a40507af0e5fd9837cd3da072de94b810))

### Miscellaneous

- Feat/set cpu limit ([#529](https://github.com/shuttle-hq/shuttle/issues/529)) - ([5c9487d](https://github.com/shuttle-hq/shuttle/commit/5c9487d0a8eaafdb6d4c002e993ca13a5635f1b6))
- Hacking static folders ([#524](https://github.com/shuttle-hq/shuttle/issues/524)) - ([84250da](https://github.com/shuttle-hq/shuttle/commit/84250dadaf59a3c7cebcf045fa577cafa9131090))
- Feat/set examples submodule to main ([#520](https://github.com/shuttle-hq/shuttle/issues/520)) - ([bc7b339](https://github.com/shuttle-hq/shuttle/commit/bc7b339cf834b8c434e1406ec05951fec686ef05))
- Interactive init gif ([#519](https://github.com/shuttle-hq/shuttle/issues/519)) - ([a957063](https://github.com/shuttle-hq/shuttle/commit/a957063784a21ab0c48ebf7c7612045bf3f470bb))
- Improve contributing documentation ([#499](https://github.com/shuttle-hq/shuttle/issues/499)) - ([c2fa52c](https://github.com/shuttle-hq/shuttle/commit/c2fa52cee9394c3fc377e4b17fd39d03dd84890e))
- Fix command to prime database with docker-compose ([#502](https://github.com/shuttle-hq/shuttle/issues/502)) - ([80f8e12](https://github.com/shuttle-hq/shuttle/commit/80f8e12fcba85c26bcfe4aba064c73c1550f9715))
- Configurable deployment tags ([#486](https://github.com/shuttle-hq/shuttle/issues/486)) - ([ac06f5c](https://github.com/shuttle-hq/shuttle/commit/ac06f5cec2f02c11a8e816165d4265e1b737451b))
- Deployer freezes ([#478](https://github.com/shuttle-hq/shuttle/issues/478)) - ([c3c0ced](https://github.com/shuttle-hq/shuttle/commit/c3c0ceda61cdb7f00135208385fdeb6461a2e1be))
- Fixed Links in Readme ([#477](https://github.com/shuttle-hq/shuttle/issues/477)) - ([836c5f7](https://github.com/shuttle-hq/shuttle/commit/836c5f73e3cec9791ef3f5ed1d9e78ea2ef1e975))
- WIP feat: count recent start events before restart ([#469](https://github.com/shuttle-hq/shuttle/issues/469)) - ([3a98a47](https://github.com/shuttle-hq/shuttle/commit/3a98a479ce24b5b4076b44e0180a0049625c7ef0))
- Revive via gateway endpoint ([#460](https://github.com/shuttle-hq/shuttle/issues/460)) - ([88c877d](https://github.com/shuttle-hq/shuttle/commit/88c877df6e94307e8271ec7972704dfc8f3bb19c))
- Remove old migrator ([#463](https://github.com/shuttle-hq/shuttle/issues/463)) - ([83cbccd](https://github.com/shuttle-hq/shuttle/commit/83cbccd70325c1da9d2afb15b2ba9f2aa82e3656))
- Add more helpful flags to Makefile - ([617bad0](https://github.com/shuttle-hq/shuttle/commit/617bad0357c9513b55bfa270f8c80593eecd64d5))
- Add docker-compose extra flags param in Makefile ([#446](https://github.com/shuttle-hq/shuttle/issues/446)) - ([c2499cb](https://github.com/shuttle-hq/shuttle/commit/c2499cb21b333ced3eac6f43f410d2102af0010e))
- Merge remote-tracking branch 'upstream/main' into development - ([7bfb1a2](https://github.com/shuttle-hq/shuttle/commit/7bfb1a2edd70e599358451e3309c5db1389f0e2e))

## [0.7.2](https://github.com/shuttle-hq/shuttle/compare/v0.7.1..v0.7.2) - 2022-10-28

### Features

- *(www)* Beta blog updates ([#434](https://github.com/shuttle-hq/shuttle/issues/434)) - ([4be3e5a](https://github.com/shuttle-hq/shuttle/commit/4be3e5a09f4d4e72ca4bcd72db1487d8414eebdf))
- Link the tracing spans between services ([#445](https://github.com/shuttle-hq/shuttle/issues/445)) - ([c4f0837](https://github.com/shuttle-hq/shuttle/commit/c4f08374b1751a84491391e5be5171b5ab014770))
- Replace cursed gif ([#441](https://github.com/shuttle-hq/shuttle/issues/441)) - ([c7a13f9](https://github.com/shuttle-hq/shuttle/commit/c7a13f977e8e68a51d1e584953b240da83d57ed2))
- Add captioned image component ([#440](https://github.com/shuttle-hq/shuttle/issues/440)) - ([61987e2](https://github.com/shuttle-hq/shuttle/commit/61987e2a25097cbcdb8bb0d055a4f5325b6a05f5))

### Refactor

- Do our own health checks on deployer containers ([#427](https://github.com/shuttle-hq/shuttle/issues/427)) - ([fb623e7](https://github.com/shuttle-hq/shuttle/commit/fb623e786ef5df76bcde61cecdd8f24cf0caf6ac))
- Provide better context for errors ([#430](https://github.com/shuttle-hq/shuttle/issues/430)) - ([bc13eb7](https://github.com/shuttle-hq/shuttle/commit/bc13eb7effd452d94fa2aee5d8608f1f0d7795aa))

### Miscellaneous Tasks

- V0.7.2 ([#442](https://github.com/shuttle-hq/shuttle/issues/442)) - ([b097d3b](https://github.com/shuttle-hq/shuttle/commit/b097d3be67281910786afb444f9918d6c2894f23))

### Miscellaneous

- Merge pull request #443 from shuttle-hq/development - ([5fd6e40](https://github.com/shuttle-hq/shuttle/commit/5fd6e4086e9fed7ce58d0b9912d080ab7db88e02))
- Merge remote-tracking branch 'upstream/main' into development - ([566b9e8](https://github.com/shuttle-hq/shuttle/commit/566b9e811fe9246ceb98645c76e15ba1779cc7d8))
- Feat/update contributing ([#426](https://github.com/shuttle-hq/shuttle/issues/426)) - ([b430d21](https://github.com/shuttle-hq/shuttle/commit/b430d2180790ff58ae5614746b0544aa4bf023a5))
- Post small tweaks ([#439](https://github.com/shuttle-hq/shuttle/issues/439)) - ([a5b7634](https://github.com/shuttle-hq/shuttle/commit/a5b763462e7ebe90f8922c5dd9393de0e51a3ca2))
- Added some images ([#435](https://github.com/shuttle-hq/shuttle/issues/435)) - ([f05fc2b](https://github.com/shuttle-hq/shuttle/commit/f05fc2b1e2492171228c8411c1c694c657bb2613))
- Clear build folder before extracting ([#428](https://github.com/shuttle-hq/shuttle/issues/428)) - ([0968b72](https://github.com/shuttle-hq/shuttle/commit/0968b72916b35c866fe9954edb8ad2911b6c38f9))

## [0.7.1](https://github.com/shuttle-hq/shuttle/compare/v0.7.0..v0.7.1) - 2022-10-24

### Features

- *(blog)* Add missing sqlx migration code to auth blog post ([#408](https://github.com/shuttle-hq/shuttle/issues/408)) - ([cf1b30c](https://github.com/shuttle-hq/shuttle/commit/cf1b30ca92b002550448ff67867b351894eb389e))
- *(deployer)* Implement container memory limits ([#411](https://github.com/shuttle-hq/shuttle/issues/411)) - ([607c3e1](https://github.com/shuttle-hq/shuttle/commit/607c3e1590b7f0b6136c6b7d094e4cc71aaabd90))
- *(deployer)* Add support for building wasm projects ([#437](https://github.com/shuttle-hq/shuttle/issues/437)) - ([67a4e91](https://github.com/shuttle-hq/shuttle/commit/67a4e91f5a1d50a268e131a31bdbaaea1584b328))
- *(next)* Expand macro into axum routes ([#488](https://github.com/shuttle-hq/shuttle/issues/488)) - ([c2b0f63](https://github.com/shuttle-hq/shuttle/commit/c2b0f63d7b58847a36c82a3a9c229cb01b6b09e2))
- *(shuttle-next)* First edition of axum-wasm router ([#472](https://github.com/shuttle-hq/shuttle/issues/472)) - ([019764e](https://github.com/shuttle-hq/shuttle/commit/019764eba6f4bcea36ab76be207199c60bc598a6))
- Shell completions ([#343](https://github.com/shuttle-hq/shuttle/issues/343)) - ([9c83baf](https://github.com/shuttle-hq/shuttle/commit/9c83baf9eebea5a8c31de976e03fcc9c2add096d))
- Gateway admin revive ([#412](https://github.com/shuttle-hq/shuttle/issues/412)) - ([6e771c7](https://github.com/shuttle-hq/shuttle/commit/6e771c7d8c0d000aeffc93d1698db2f4b66b5140))
- Deployer next ([#575](https://github.com/shuttle-hq/shuttle/issues/575)) - ([cc072b2](https://github.com/shuttle-hq/shuttle/commit/cc072b2bb392126c075aa80bf03fd482ddda4f6d))
- Get runtime binary from cargo install ([#578](https://github.com/shuttle-hq/shuttle/issues/578)) - ([b17b3a1](https://github.com/shuttle-hq/shuttle/commit/b17b3a19193779a63a6cbe914aaeb71877ede03b))
- DX ([#577](https://github.com/shuttle-hq/shuttle/issues/577)) - ([39c9d1c](https://github.com/shuttle-hq/shuttle/commit/39c9d1cdef9345d9d0e886a394ab2b9b9da55293))
- Change log read to spawn blocking ([#574](https://github.com/shuttle-hq/shuttle/issues/574)) - ([509e373](https://github.com/shuttle-hq/shuttle/commit/509e373e4224b33bcbf419e6b5c831002ecb5b5e))
- Hook in runtime logs ([#568](https://github.com/shuttle-hq/shuttle/issues/568)) - ([5b033d0](https://github.com/shuttle-hq/shuttle/commit/5b033d0ba27b9c976e1051df224f224138aee513))
- Refactor router and router inner ([#566](https://github.com/shuttle-hq/shuttle/issues/566)) - ([8324824](https://github.com/shuttle-hq/shuttle/commit/832482430524860258403feac2f94abe8cfe2d40))
- Embed runtime into client and deployer ([#559](https://github.com/shuttle-hq/shuttle/issues/559)) - ([c34d5e4](https://github.com/shuttle-hq/shuttle/commit/c34d5e493e6583e697ca0a9718102ee3e49d8690))
- Return streaming body from wasm router ([#558](https://github.com/shuttle-hq/shuttle/issues/558)) - ([9db7f90](https://github.com/shuttle-hq/shuttle/commit/9db7f90293142f4c1b716b3502f20634fca8cea6))
- Merge main into shuttle-next ([#543](https://github.com/shuttle-hq/shuttle/issues/543)) - ([b6e668b](https://github.com/shuttle-hq/shuttle/commit/b6e668bbf3c7d2b4e202bee07474dec0e6520293))
- Get logs from runtime ([#459](https://github.com/shuttle-hq/shuttle/issues/459)) - ([ee342e4](https://github.com/shuttle-hq/shuttle/commit/ee342e49b9530f7d9d372f77ff28b87d6857973d))
- Move factory to runtime ([#444](https://github.com/shuttle-hq/shuttle/issues/444)) - ([5546fb2](https://github.com/shuttle-hq/shuttle/commit/5546fb290fd0e0610982ec60dbf7a0b357ebcb6d))
- Create a control plane interface (part 1) ([#436](https://github.com/shuttle-hq/shuttle/issues/436)) - ([37ade4c](https://github.com/shuttle-hq/shuttle/commit/37ade4ca7f82bc681b7ff19863a430ea0865587d))
- Add   --provisioner-address arg to both runtimes ([#433](https://github.com/shuttle-hq/shuttle/issues/433)) - ([e773225](https://github.com/shuttle-hq/shuttle/commit/e773225441a3f15b9b35db30c3623d821f4e6b9e))
- Shuttle-serenity initial commit poc ([#429](https://github.com/shuttle-hq/shuttle/issues/429)) - ([a1c5fc5](https://github.com/shuttle-hq/shuttle/commit/a1c5fc5ffc3faede4b6bc88fd2ac4445b0a4a87c))

### Bug Fixes

- *(cargo-shuttle)* Prevent crash when config owned by root ([#409](https://github.com/shuttle-hq/shuttle/issues/409)) - ([37755ca](https://github.com/shuttle-hq/shuttle/commit/37755ca94bd24d569e874c7c756f9baad6eedc9e))
- Use correct timeout start point ([#410](https://github.com/shuttle-hq/shuttle/issues/410)) - ([76f4fee](https://github.com/shuttle-hq/shuttle/commit/76f4fee5bec8df7af3057c20ae3559193df3b652))
- Missing feature flag in common dep ([#573](https://github.com/shuttle-hq/shuttle/issues/573)) - ([a660b15](https://github.com/shuttle-hq/shuttle/commit/a660b155a42d732dfcf7635cd55af9e8e8333fc5))

### Refactor

- Remove the serenity runtime ([#549](https://github.com/shuttle-hq/shuttle/issues/549)) - ([c4dd391](https://github.com/shuttle-hq/shuttle/commit/c4dd3912e7642d6938a4cf93d1aa97bbd243a4f1))
- One store per request ([#510](https://github.com/shuttle-hq/shuttle/issues/510)) - ([a4ef6c3](https://github.com/shuttle-hq/shuttle/commit/a4ef6c335e0bd676bfa961a2782d2827d3acbfc9))
- Combine runtimes into one ([#438](https://github.com/shuttle-hq/shuttle/issues/438)) - ([da46e60](https://github.com/shuttle-hq/shuttle/commit/da46e602c684548cfd0251b67a46ae7773ae99bc))
- Create runtimes workspace ([#432](https://github.com/shuttle-hq/shuttle/issues/432)) - ([f6e1766](https://github.com/shuttle-hq/shuttle/commit/f6e17660e2334760fcd55cba58009a7743c21413))

### Miscellaneous Tasks

- *(shuttle-next)* Stop runtime services ([#481](https://github.com/shuttle-hq/shuttle/issues/481)) - ([f913b8a](https://github.com/shuttle-hq/shuttle/commit/f913b8a7f7f46bcf8908e77cbe939c23620219a9))
- *(www)* Shuttle beta signup ([#421](https://github.com/shuttle-hq/shuttle/issues/421)) - ([1b983e8](https://github.com/shuttle-hq/shuttle/commit/1b983e8c7e0dd2b4325161785664f38799de7e89))
- V0.7.1 ([#424](https://github.com/shuttle-hq/shuttle/issues/424)) - ([65b71c8](https://github.com/shuttle-hq/shuttle/commit/65b71c8918b9fdce55e8b48a7607c08e2b82a964))

### Miscellaneous

- Big archives being cut off at 32 768 bytes ([#423](https://github.com/shuttle-hq/shuttle/issues/423)) - ([b00671d](https://github.com/shuttle-hq/shuttle/commit/b00671dc32b341ad6dafba0e7ac1fb1b803a68f5))
- Package Secrets.toml ([#422](https://github.com/shuttle-hq/shuttle/issues/422)) - ([c222354](https://github.com/shuttle-hq/shuttle/commit/c2223541c64326c6762e5e82035a1d49fe2049f9))
- Fix thruster postgres example ([#414](https://github.com/shuttle-hq/shuttle/issues/414)) - ([7c05afc](https://github.com/shuttle-hq/shuttle/commit/7c05afc2588cdaaf0dcf22180c8c71d01f95de91))
- Article/beta article ([#420](https://github.com/shuttle-hq/shuttle/issues/420)) - ([b4149f2](https://github.com/shuttle-hq/shuttle/commit/b4149f2f0e25d0f27b11ba36cd93aaeb24f0022c))
- Timeout curl health check on deployer ([#415](https://github.com/shuttle-hq/shuttle/issues/415)) - ([8f7a341](https://github.com/shuttle-hq/shuttle/commit/8f7a341ec862b2b065ba5667c45d524d29b410cb))
- `transport error` when trying to connect to provisioner ([#416](https://github.com/shuttle-hq/shuttle/issues/416)) - ([e676715](https://github.com/shuttle-hq/shuttle/commit/e676715b3e97ecf6ef59b9ec80afccf9b989b268))
- 0.7.0 ([#407](https://github.com/shuttle-hq/shuttle/issues/407)) - ([ac43016](https://github.com/shuttle-hq/shuttle/commit/ac430161f85b681fb8cdef336845c6368130867c))
- Merge main ([#572](https://github.com/shuttle-hq/shuttle/issues/572)) - ([9697090](https://github.com/shuttle-hq/shuttle/commit/9697090557cd70c7e9a4abffa525bd3e225758b9))
- Expanded broken merge ([#567](https://github.com/shuttle-hq/shuttle/issues/567)) - ([d7ff85f](https://github.com/shuttle-hq/shuttle/commit/d7ff85f445d07936854059ee0d27800f1606eb0e))
- Shuttle next bump deps ([#551](https://github.com/shuttle-hq/shuttle/issues/551)) - ([1487ddf](https://github.com/shuttle-hq/shuttle/commit/1487ddfe871bbe751ce8c75a3278dc51d8ffd616))
- Parse shuttle::endpoint macro ([#490](https://github.com/shuttle-hq/shuttle/issues/490)) - ([16abe40](https://github.com/shuttle-hq/shuttle/commit/16abe40d109a2361bfb5313a5f73d1f9170c1348))
- WIP feat: start runtime from deployer ([#450](https://github.com/shuttle-hq/shuttle/issues/450)) - ([0e8ce8b](https://github.com/shuttle-hq/shuttle/commit/0e8ce8b6772cc6f3f7eba4fcac96bbf8a77d8afb))
- Shuttle next wrapper POC ([#431](https://github.com/shuttle-hq/shuttle/issues/431)) - ([f7e09b6](https://github.com/shuttle-hq/shuttle/commit/f7e09b6ac594e813668fdfdf15f1ddd6d0306c00))

## [0.7.0](https://github.com/shuttle-hq/shuttle/compare/v0.5.2..v0.7.0) - 2022-10-17

### Features

- *(0.6.0)* Update cargo shuttle init generated code ([#392](https://github.com/shuttle-hq/shuttle/issues/392)) - ([1f80b75](https://github.com/shuttle-hq/shuttle/commit/1f80b752afeba978c9bf28e1790ef9102e81b1a1))
- *(cargo-shuttle)* Better client errors ([#394](https://github.com/shuttle-hq/shuttle/issues/394)) - ([b5709fa](https://github.com/shuttle-hq/shuttle/commit/b5709fa139285b2f3d7e4bda70e336d5d7a36463))
- *(cargo-shuttle)* Better error messages ([#391](https://github.com/shuttle-hq/shuttle/issues/391)) - ([37460c3](https://github.com/shuttle-hq/shuttle/commit/37460c35050adabe754859e19fd8be5cf4f4462b))
- *(common)* Clean up deps passed to user crates ([#355](https://github.com/shuttle-hq/shuttle/issues/355)) - ([e7a1494](https://github.com/shuttle-hq/shuttle/commit/e7a1494a05de42b939d85cfad1abe012ec702ecb))
- *(deployer)* Split up deployer error enum ([#339](https://github.com/shuttle-hq/shuttle/issues/339)) - ([d4bf86c](https://github.com/shuttle-hq/shuttle/commit/d4bf86c552a9c5013b845c3933f3195dadfea547))
- *(gateway)* Initial commit - ([ec293a0](https://github.com/shuttle-hq/shuttle/commit/ec293a047fc08fe9269d8ed59f06dbfea67ec0af))
- *(service)* Add thruster framework as service ([#389](https://github.com/shuttle-hq/shuttle/issues/389)) - ([d8180d8](https://github.com/shuttle-hq/shuttle/commit/d8180d8b3d14d82c4b044f8203adb7443ea5d59f))
- *(service)* Integrate salvo support ([#334](https://github.com/shuttle-hq/shuttle/issues/334)) - ([60011c4](https://github.com/shuttle-hq/shuttle/commit/60011c41b59e9e937fcabb8335b16f1268226135))
- *(tracing)* Auto register tracing layer ([#324](https://github.com/shuttle-hq/shuttle/issues/324)) - ([70f4784](https://github.com/shuttle-hq/shuttle/commit/70f47845a9e00d07c7fdc3781501cd421cbf0568))
- *(www)* Replace mixpanel with google analytics ([#345](https://github.com/shuttle-hq/shuttle/issues/345)) - ([2634ce6](https://github.com/shuttle-hq/shuttle/commit/2634ce62cdb191e0fa8a79bf995174a2e8b059cc))
- *(www/docs)* Update blog and secrets readme ([#402](https://github.com/shuttle-hq/shuttle/issues/402)) - ([c2d7fcc](https://github.com/shuttle-hq/shuttle/commit/c2d7fcc86abdcd559828c26524b3eae9d2a2c28c))
- Build deploys in release mode ([#403](https://github.com/shuttle-hq/shuttle/issues/403)) - ([b4e6aea](https://github.com/shuttle-hq/shuttle/commit/b4e6aeaacce499d94f82ac6696d8d55945648551))
- Added publish: false in generated Cargo.toml to avoid accidental cargo publish ([#358](https://github.com/shuttle-hq/shuttle/issues/358)) - ([2d67c6b](https://github.com/shuttle-hq/shuttle/commit/2d67c6b7cd71b6590718d83f95b2a2ffdf3f0c0a))
- Update docs ([#396](https://github.com/shuttle-hq/shuttle/issues/396)) - ([560f985](https://github.com/shuttle-hq/shuttle/commit/560f985fea4ed79ffe2ae00782117122d7be0f28))
- Add projects to small migrator ([#383](https://github.com/shuttle-hq/shuttle/issues/383)) - ([26613d9](https://github.com/shuttle-hq/shuttle/commit/26613d968a7ed41a54ab06b54759864c2d6f3ed6))
- Small migrator for generating migration sql ([#378](https://github.com/shuttle-hq/shuttle/issues/378)) - ([c6a6fb1](https://github.com/shuttle-hq/shuttle/commit/c6a6fb1a21850d12fc359c04868248bef7812c1a))
- (gd) version check on server side ([#377](https://github.com/shuttle-hq/shuttle/issues/377)) - ([8c4d913](https://github.com/shuttle-hq/shuttle/commit/8c4d913c0fd3a6e82a2a1daeaefb7acc98b7515a))
- G&D shutdown on build ([#369](https://github.com/shuttle-hq/shuttle/issues/369)) - ([47a4b70](https://github.com/shuttle-hq/shuttle/commit/47a4b70af3ef0f1abed16e1542fdf438340143e4))
- Deleting a project on gateway frees it for good ([#364](https://github.com/shuttle-hq/shuttle/issues/364)) - ([acc345e](https://github.com/shuttle-hq/shuttle/commit/acc345e6f681c3e66ec761ea92a740185d01c275))
- Gateway init ([#363](https://github.com/shuttle-hq/shuttle/issues/363)) - ([d434f19](https://github.com/shuttle-hq/shuttle/commit/d434f1903014ab781cc8b83f78068582a54ea4bc))
- Deployer proxy ([#347](https://github.com/shuttle-hq/shuttle/issues/347)) - ([fbc15da](https://github.com/shuttle-hq/shuttle/commit/fbc15da482d396e588f7f2374789159ba839d011))
- Deployer users ([#327](https://github.com/shuttle-hq/shuttle/issues/327)) - ([092e55e](https://github.com/shuttle-hq/shuttle/commit/092e55e15ab562e4486cefa890d46a9a3beafd5a))
- Deployer secrets ([#321](https://github.com/shuttle-hq/shuttle/issues/321)) - ([9375b4a](https://github.com/shuttle-hq/shuttle/commit/9375b4a69e072dc1c521ea42e701d326283a05e4))
- Deployer client updates ([#298](https://github.com/shuttle-hq/shuttle/issues/298)) - ([d93efc4](https://github.com/shuttle-hq/shuttle/commit/d93efc4540d2ebe19abc3f1a8ceea0398d0100da))
- Telemetery ([#271](https://github.com/shuttle-hq/shuttle/issues/271)) - ([9421fee](https://github.com/shuttle-hq/shuttle/commit/9421feeff1429156bae7d4710bed7f151277422c))
- Deployer load and run ([#235](https://github.com/shuttle-hq/shuttle/issues/235)) - ([98f1182](https://github.com/shuttle-hq/shuttle/commit/98f1182b4ab854c45b411ee02b4d63e704b85d02))
- Deployer build logs ([#265](https://github.com/shuttle-hq/shuttle/issues/265)) - ([b951101](https://github.com/shuttle-hq/shuttle/commit/b9511015a88fa5c9235dfe5ef50eee9beee2ac47))
- Deployer registry cache ([#259](https://github.com/shuttle-hq/shuttle/issues/259)) - ([6ae2e56](https://github.com/shuttle-hq/shuttle/commit/6ae2e56a28511278a7b999bf5b77067ee111d63f))
- Run a service's unit tests on server side before loading and executing ([#227](https://github.com/shuttle-hq/shuttle/issues/227)) - ([0c6eb75](https://github.com/shuttle-hq/shuttle/commit/0c6eb754b13d014fac939a8fca126183974f95c1))
- Deployer log table ([#221](https://github.com/shuttle-hq/shuttle/issues/221)) - ([1bf0e7f](https://github.com/shuttle-hq/shuttle/commit/1bf0e7f351dff880bbeedf78df5837deb3bb7225))
- Build incoming services ([#220](https://github.com/shuttle-hq/shuttle/issues/220)) - ([9e98cea](https://github.com/shuttle-hq/shuttle/commit/9e98cea0b4449a2ed357b1252a163d541ebacf54))
- Deployer service skeleton ([#215](https://github.com/shuttle-hq/shuttle/issues/215)) - ([120887a](https://github.com/shuttle-hq/shuttle/commit/120887a7e16ccfa330c681abe86166e64a7e8936))
- Resource attribute options ([#276](https://github.com/shuttle-hq/shuttle/issues/276)) - ([4bdfdba](https://github.com/shuttle-hq/shuttle/commit/4bdfdba6007f6802ca28e4fe5eb4d690049ab8f0))

### Bug Fixes

- No panic in main on startup ([#390](https://github.com/shuttle-hq/shuttle/issues/390)) - ([67d0e2e](https://github.com/shuttle-hq/shuttle/commit/67d0e2eb9e96cb9871a3f3716b26a142346925f8))
- Gateway args in docker compose ([#366](https://github.com/shuttle-hq/shuttle/issues/366)) - ([da5cc28](https://github.com/shuttle-hq/shuttle/commit/da5cc28ea28e019a3f85ff01f42d9e385ab97b46))
- Fix rocket config ([#351](https://github.com/shuttle-hq/shuttle/issues/351)) - ([0dc7774](https://github.com/shuttle-hq/shuttle/commit/0dc77744b331406c223f84f08512ef023f159c5d))
- Server side check for service version before deploying ([#214](https://github.com/shuttle-hq/shuttle/issues/214)) - ([260098a](https://github.com/shuttle-hq/shuttle/commit/260098a8c4e34783d022b15caed88c6abb2e0a26))

### Refactor

- *(0.6.0rc1)* Clippy and fmt ([#380](https://github.com/shuttle-hq/shuttle/issues/380)) - ([280197d](https://github.com/shuttle-hq/shuttle/commit/280197d7a759d4270cc80593b141b34669a1a87a))
- Change join! in main to select! ([#376](https://github.com/shuttle-hq/shuttle/issues/376)) - ([7f0cd09](https://github.com/shuttle-hq/shuttle/commit/7f0cd0958ee584a648d2a2378cce88adb3416a36))
- Remove mutex in gateway sender ([#371](https://github.com/shuttle-hq/shuttle/issues/371)) - ([1b374c1](https://github.com/shuttle-hq/shuttle/commit/1b374c1ce77acf5a51471801e7ab5c7d490a39c7))
- Remove user management from deployer ([#356](https://github.com/shuttle-hq/shuttle/issues/356)) - ([45d4976](https://github.com/shuttle-hq/shuttle/commit/45d4976cd3af5136c7aff71734f2bb084073d201))
- Remove lazy_static - ([845cb79](https://github.com/shuttle-hq/shuttle/commit/845cb79ca08300697bde326050680e5cd9b08be3))
- G&D project config ([#353](https://github.com/shuttle-hq/shuttle/issues/353)) - ([0473700](https://github.com/shuttle-hq/shuttle/commit/047370091153be0f1bd4d3e359e31e4e00212582))
- Backend to deployer image - ([fd35695](https://github.com/shuttle-hq/shuttle/commit/fd356959cee078507eb67988ca86209f7d952535))
- Service to project routes - ([b616e10](https://github.com/shuttle-hq/shuttle/commit/b616e10b1e6cf2703378e10bb33a77e08e617477))
- Cargo fmt and more logs - ([641cfca](https://github.com/shuttle-hq/shuttle/commit/641cfca201b0dec37a80f74387cea9c63893e290))
- Gateway `log` to `tracing` - ([b3106ff](https://github.com/shuttle-hq/shuttle/commit/b3106ffacf8e6c0df353cbc2fa85381533bc2fda))
- Rename `project` to `service` in deployer ([#338](https://github.com/shuttle-hq/shuttle/issues/338)) - ([4e00a03](https://github.com/shuttle-hq/shuttle/commit/4e00a0382380b6c590ffd79d388c9d2b0ed9fac8))
- Deployer db ([#242](https://github.com/shuttle-hq/shuttle/issues/242)) - ([4078ad7](https://github.com/shuttle-hq/shuttle/commit/4078ad75f226cb5666c29c77470896baa5cf80b8))
- Erroring on raw response (deployer) ([#266](https://github.com/shuttle-hq/shuttle/issues/266)) - ([f8e0ab8](https://github.com/shuttle-hq/shuttle/commit/f8e0ab84a42db8959aa28859414c050df28c1281))
- Drop api crate ([#229](https://github.com/shuttle-hq/shuttle/issues/229)) - ([8f51cd4](https://github.com/shuttle-hq/shuttle/commit/8f51cd41d31c615632bc53f5259cb6bd443e1083))
- Plugins ([#273](https://github.com/shuttle-hq/shuttle/issues/273)) - ([9e2c01f](https://github.com/shuttle-hq/shuttle/commit/9e2c01fdfcfd514ea741efff884f554433a0aaef))

### Testing

- Fixes hanging tests ([#374](https://github.com/shuttle-hq/shuttle/issues/374)) - ([638dffc](https://github.com/shuttle-hq/shuttle/commit/638dffcd35806e41cc36b069ad687913d3931ba8))

### Miscellaneous Tasks

- V0.7.0 ([#404](https://github.com/shuttle-hq/shuttle/issues/404)) - ([5d2215b](https://github.com/shuttle-hq/shuttle/commit/5d2215b6d880efe2e0428aac1d3af75096aecbb2))
- Error logs ([#401](https://github.com/shuttle-hq/shuttle/issues/401)) - ([1b4e8ab](https://github.com/shuttle-hq/shuttle/commit/1b4e8abcb93ca733cebf152bbee85d32cebe0c19))
- V0.6.0 ([#397](https://github.com/shuttle-hq/shuttle/issues/397)) - ([16373b2](https://github.com/shuttle-hq/shuttle/commit/16373b20ffea79187d164e306644d463749d25e7))
- Go all green ([#388](https://github.com/shuttle-hq/shuttle/issues/388)) - ([c69d72d](https://github.com/shuttle-hq/shuttle/commit/c69d72d974594eb7a8592389a4e6feab65af813c))
- Add missing tests for persist and missing clippy for DBs ([#349](https://github.com/shuttle-hq/shuttle/issues/349)) - ([5559b93](https://github.com/shuttle-hq/shuttle/commit/5559b9326c7902d42a7ac5447df423db1a6573a2))
- Persistant Storage ([#306](https://github.com/shuttle-hq/shuttle/issues/306)) - ([dbfb0ee](https://github.com/shuttle-hq/shuttle/commit/dbfb0eec7fc3d52c84db18e48e30d1ed2e029d69))

### Miscellaneous

- Merge pull request #400 from shuttle-hq/v0.6.0rc1 - ([c22948c](https://github.com/shuttle-hq/shuttle/commit/c22948ced3b49cad011149d5283fe0a41c951e6e))
- Merge remote-tracking branch 'upstream/main' into v0.6.0rc1 - ([dff63bb](https://github.com/shuttle-hq/shuttle/commit/dff63bbe98ad91741601a5a530df379ddb32918c))
- Migration ([#395](https://github.com/shuttle-hq/shuttle/issues/395)) - ([2f6e2dd](https://github.com/shuttle-hq/shuttle/commit/2f6e2dd005fb81ed6171273d8c7d1f17faea20bf))
- D&G ([#393](https://github.com/shuttle-hq/shuttle/issues/393)) - ([43791af](https://github.com/shuttle-hq/shuttle/commit/43791afffd83d0b76d77c37571aeb2f6eccc01d4))
- Networking fixes ([#386](https://github.com/shuttle-hq/shuttle/issues/386)) - ([893d074](https://github.com/shuttle-hq/shuttle/commit/893d07434dee4d50b14feb1e3ff100499c92289a))
- Merge remote-tracking branch 'upstream/main' into v0.6.0rc1 - ([c7d2c13](https://github.com/shuttle-hq/shuttle/commit/c7d2c131b8a88d8b6a0a546832089dcee7817819))
- Store deployer states on mounted volume path ([#387](https://github.com/shuttle-hq/shuttle/issues/387)) - ([13849dd](https://github.com/shuttle-hq/shuttle/commit/13849ddc467badd92f5e0686849e5e45c3c68445))
- Add patch to allow deployer to start services ([#385](https://github.com/shuttle-hq/shuttle/issues/385)) - ([fabf1f1](https://github.com/shuttle-hq/shuttle/commit/fabf1f1548932996fc2e8a6baa19f276c5e6a75c))
- Add auth hooks ([#379](https://github.com/shuttle-hq/shuttle/issues/379)) - ([9ab2a5a](https://github.com/shuttle-hq/shuttle/commit/9ab2a5acb3836362f9d568c80d9fa0a68b751e71))
- Allow attaching to network ([#384](https://github.com/shuttle-hq/shuttle/issues/384)) - ([efd7a6a](https://github.com/shuttle-hq/shuttle/commit/efd7a6a02782475d03cfc16f64fa4ae6b1e5d06a))
- Added health check ([#373](https://github.com/shuttle-hq/shuttle/issues/373)) - ([24ffa1a](https://github.com/shuttle-hq/shuttle/commit/24ffa1acdbf0f66b7f2967599abf3f2b8328853b))
- Second deployment failing ([#375](https://github.com/shuttle-hq/shuttle/issues/375)) - ([170c2ec](https://github.com/shuttle-hq/shuttle/commit/170c2ec9069926e5206c8e2c208c3e05fba1e5eb))
- Merge remote-tracking branch 'upstream/main' into v0.6.0rc1 - ([177e4d9](https://github.com/shuttle-hq/shuttle/commit/177e4d9053f1286b16129552e7642879e2460536))
- Dev deployment for v0.6.0rc1 with gateway ([#362](https://github.com/shuttle-hq/shuttle/issues/362)) - ([c5cf692](https://github.com/shuttle-hq/shuttle/commit/c5cf692a67b5a78c3f99a4778d5234570b01f0c4))
- Cargo sort - ([89ca387](https://github.com/shuttle-hq/shuttle/commit/89ca3879b289cc5f8b4d1047a9ede32be1e4a8a4))
- Pin dev environment to Rust v1.63.0 - ([fe24117](https://github.com/shuttle-hq/shuttle/commit/fe241179508a1e07a96811c108cc3a5cc9c9bd9c))
- Merge remote-tracking branch 'brokad/feat/gateway' into v0.6.0rc1 - ([3f247d3](https://github.com/shuttle-hq/shuttle/commit/3f247d3864f86fe4f9450c5d6e0ec70cb7495e16))
- Merge branch 'main' into feat/gateway - ([055bda8](https://github.com/shuttle-hq/shuttle/commit/055bda86cd065ee1fe44f14d43ecaecf7ca8e57d))
- Compat v0.3 - ([bbfafa6](https://github.com/shuttle-hq/shuttle/commit/bbfafa66f27692f53de474ece06e674aa1c1e66b))
- Fmt - ([105ccc9](https://github.com/shuttle-hq/shuttle/commit/105ccc9df79b976b058759a934e23f1d15a1725e))
- Tweaks - ([f223cf9](https://github.com/shuttle-hq/shuttle/commit/f223cf98bd918e6ecbfe04ca52d34dda87718594))
- Fmt - ([4310259](https://github.com/shuttle-hq/shuttle/commit/4310259587be015340c2a66d425f0b39ea4cd7b9))
- Tweaks - ([6aaebd1](https://github.com/shuttle-hq/shuttle/commit/6aaebd10e37e39819fd94cb55424d3e898cf4acf))
- Fmt - ([54d0d5c](https://github.com/shuttle-hq/shuttle/commit/54d0d5caa80ea7aece26d4a8bae4ece8a3b57e4c))
- Tweaks - ([cf6079b](https://github.com/shuttle-hq/shuttle/commit/cf6079b7c9cab280ef576845a07f98417c11e35b))
- Fix fix fix - ([e2cbaea](https://github.com/shuttle-hq/shuttle/commit/e2cbaeaff2ba12b9a165a6cf7fb6b7f29efe90dc))
- Fix fix - ([bdabb6a](https://github.com/shuttle-hq/shuttle/commit/bdabb6a0e6527bb05107e05fb34445c04ceb7f75))
- Fix - ([b551fa3](https://github.com/shuttle-hq/shuttle/commit/b551fa3f52b02edaf4bc134db9083f46fac8b561))
- Fmt round 1 - ([41ef5d6](https://github.com/shuttle-hq/shuttle/commit/41ef5d6acdfc0effbd7ed714fe24f5b21bd05fa0))
- Remarks - ([6df573c](https://github.com/shuttle-hq/shuttle/commit/6df573c80cd885dec15df93948c921c25e1c4499))
- E2E - ([2b09519](https://github.com/shuttle-hq/shuttle/commit/2b09519c453461ebc6874c41646d0774224df7c9))
- Integration test harness - ([21ef659](https://github.com/shuttle-hq/shuttle/commit/21ef65963ca33f4f434c5b8c9fbb36a02f9464fa))
- Tweaks - ([2c4a23f](https://github.com/shuttle-hq/shuttle/commit/2c4a23f3ad647e6c1044df14222f2ef0696e957f))
- Testy tests - ([501b1fb](https://github.com/shuttle-hq/shuttle/commit/501b1fbfe2e6c23cc328bdfb149bceb11513f3e7))
- Tweaks - ([ec25b7e](https://github.com/shuttle-hq/shuttle/commit/ec25b7e6b67a888963c4cc654cbfa17b20ee033f))
- More more tests - ([3f4be2c](https://github.com/shuttle-hq/shuttle/commit/3f4be2c8a3aaa52e95dece99613b72e875a3756e))
- WIP - ([f840144](https://github.com/shuttle-hq/shuttle/commit/f840144c66849d0c4edb951aedcf3992e9e49e17))
- Tweaks - ([a636514](https://github.com/shuttle-hq/shuttle/commit/a6365142264805875c6bc9bc02391e7e41dd1186))
- Some more tests - ([86ea5a2](https://github.com/shuttle-hq/shuttle/commit/86ea5a23692bf66f348a5f5380f7c4d2b5fc80e8))
- Apply suggestions from code review - ([6cea0fe](https://github.com/shuttle-hq/shuttle/commit/6cea0fe3ac675116c597ba7c4bdf4064500b021c))
- Added status test - ([15bb091](https://github.com/shuttle-hq/shuttle/commit/15bb091e72ee6c68b8e9fbb940fc6b1e5dbb1a30))
- Added timeout - ([a64a009](https://github.com/shuttle-hq/shuttle/commit/a64a009acca3445345d5f77b382dcc6a69e51032))
- More tests - ([1070535](https://github.com/shuttle-hq/shuttle/commit/107053585d8b4c1f606cc8139c7a9f61372b14a7))
- WIP - ([538abb3](https://github.com/shuttle-hq/shuttle/commit/538abb3f723f092b9f9ad06402163ca0270a174b))
- More tests - ([8cea127](https://github.com/shuttle-hq/shuttle/commit/8cea127a0498bcd2c13ee12624bdd70a72cc00c4))
- Some tests - ([d32dc7e](https://github.com/shuttle-hq/shuttle/commit/d32dc7ef0d75c3097b4aea334b8b789e2ca4d95c))
- Proxy fixings - ([2565eed](https://github.com/shuttle-hq/shuttle/commit/2565eeddb3940794e420a664a451294fc8edd630))
- Context in errors - ([7baf03e](https://github.com/shuttle-hq/shuttle/commit/7baf03e8266213c8cfb77dae699c430452f52da5))
- Fix networking with provisioner - ([1362ae5](https://github.com/shuttle-hq/shuttle/commit/1362ae5498007eb82084965f8fb2191d2a2ee9a7))
- Better handling of errors - ([9d38e46](https://github.com/shuttle-hq/shuttle/commit/9d38e4629634770aa1dfd7023134c1e64891870c))
- Temp fix for workspace issues - ([7ee18ec](https://github.com/shuttle-hq/shuttle/commit/7ee18ec396dc3fae376f8ffcd948e349a984d024))
- Bump the bollard - ([9c2252f](https://github.com/shuttle-hq/shuttle/commit/9c2252f082e732e43ff529e84da76b2ddf7878f3))
- Add args - ([f2b850a](https://github.com/shuttle-hq/shuttle/commit/f2b850ab95db6b9544c9459ed48a5802f3406bbe))
- Add delete project - ([c9a779b](https://github.com/shuttle-hq/shuttle/commit/c9a779bc530fc779a881baca7dd8bbe3c2e624f0))
- Use health checks - ([25b8337](https://github.com/shuttle-hq/shuttle/commit/25b833760246d9c30541963501c1d1e4d0cf61ef))
- More more error handling - ([11e316e](https://github.com/shuttle-hq/shuttle/commit/11e316ea94229c768b953bdf20690f38d61e4a52))
- Handle startup refresh failures - ([baecd43](https://github.com/shuttle-hq/shuttle/commit/baecd43717edb261f6a5f68ef2856b6d8bff488a))
- Actually use prefix froms Args in state transitions - ([a033e0b](https://github.com/shuttle-hq/shuttle/commit/a033e0be85d3be05a0d972c11ce20ed5f0e58d50))
- Handle more error cases, parameterise the things - ([df49894](https://github.com/shuttle-hq/shuttle/commit/df498942f00fc4093beea44fcd9a6a3fbd7177be))
- Address some comments - ([853b16e](https://github.com/shuttle-hq/shuttle/commit/853b16e99c8e2ea234abcfd4ebed3aff7a8602e9))
- Added error mgmt - ([1fec754](https://github.com/shuttle-hq/shuttle/commit/1fec754a362ad76660a71dbd94b1955d14e059ae))
- Fixed backend image startup, errors handling, fixed auth - ([d8f1cce](https://github.com/shuttle-hq/shuttle/commit/d8f1cced49a4564edffa101eecacc2fb27b9917b))
- Merge remote-tracking branch 'upstream/main' into feat/deployer - ([1b772de](https://github.com/shuttle-hq/shuttle/commit/1b772def675e906ee2769c5e1ebbf4bf16fb02c8))
- Merge remote-tracking branch 'upstream/main' into feat/deployer - ([718e1ab](https://github.com/shuttle-hq/shuttle/commit/718e1ab4a20ce904f95116616dc13f562d7faa3f))
- Merge remote-tracking branch 'upstream/main' into feat/deployer - ([23718ad](https://github.com/shuttle-hq/shuttle/commit/23718ad752c204b7383df18051f7b61a3d7dd602))
- Merge remote-tracking branch 'upstream/main' into feat/deployer - ([7afddb2](https://github.com/shuttle-hq/shuttle/commit/7afddb2a9ce9a142b4666fdbc58a5382e542667d))
- Merge remote-tracking branch 'upstream/main' into feat/deployer - ([f4f9111](https://github.com/shuttle-hq/shuttle/commit/f4f91115690ccfe4b4d59df3d55315942781a855))
- Add auth hooks ([#379](https://github.com/shuttle-hq/shuttle/issues/379)) ([#399](https://github.com/shuttle-hq/shuttle/issues/399)) - ([e63990f](https://github.com/shuttle-hq/shuttle/commit/e63990fa666afe2080e22eb9b64840f55bc98732))
- Feat/warp support ([#326](https://github.com/shuttle-hq/shuttle/issues/326)) - ([839e6e6](https://github.com/shuttle-hq/shuttle/commit/839e6e67315f3bb8eb40b024721944001f815fd0))
- Updated docs url ([#382](https://github.com/shuttle-hq/shuttle/issues/382)) - ([0f656a0](https://github.com/shuttle-hq/shuttle/commit/0f656a0e0c71f77b273635c7f586948b37758a09))
- Updated readme ([#381](https://github.com/shuttle-hq/shuttle/issues/381)) - ([f388981](https://github.com/shuttle-hq/shuttle/commit/f388981e86e36792f7ec1ae46829be685df16f1c))
- Missing resources readme ([#365](https://github.com/shuttle-hq/shuttle/issues/365)) - ([d6163c5](https://github.com/shuttle-hq/shuttle/commit/d6163c54620bbf5885d788bbf77c887a8f1d8a2a))
- Typo fix ([#361](https://github.com/shuttle-hq/shuttle/issues/361)) - ([42ffd7a](https://github.com/shuttle-hq/shuttle/commit/42ffd7a4356f383887a2442ad1cf427ee287eef1))
- Fix/update example readme ([#360](https://github.com/shuttle-hq/shuttle/issues/360)) - ([412b84e](https://github.com/shuttle-hq/shuttle/commit/412b84e386a70d2051a2a34b5aa9cfa09b95712f))
- Added additional sentences under contribution, added examples, .. ([#359](https://github.com/shuttle-hq/shuttle/issues/359)) - ([f1ee15e](https://github.com/shuttle-hq/shuttle/commit/f1ee15e23d51fdf6e5bedd143b60036eeb601f4f))
- Segmentation fault ([#348](https://github.com/shuttle-hq/shuttle/issues/348)) - ([995e1e3](https://github.com/shuttle-hq/shuttle/commit/995e1e3ee7d1b7048284b6486c818c4ede01b3f9))
- Readme touchup ([#344](https://github.com/shuttle-hq/shuttle/issues/344)) - ([f99391e](https://github.com/shuttle-hq/shuttle/commit/f99391ec35647f6a99cb27fe6d87805a2ebef33c))
- Fix/discord bot article ([#342](https://github.com/shuttle-hq/shuttle/issues/342)) - ([3e57f95](https://github.com/shuttle-hq/shuttle/commit/3e57f952debac7fd5ccddb24d53589859bd57f29))
- Fix/discord bot article ([#341](https://github.com/shuttle-hq/shuttle/issues/341)) - ([e3ae3c6](https://github.com/shuttle-hq/shuttle/commit/e3ae3c654b7bf7053f3e26fd133f894fc86884e2))
- Serenity Discord bot tutorial ([#340](https://github.com/shuttle-hq/shuttle/issues/340)) - ([a014f74](https://github.com/shuttle-hq/shuttle/commit/a014f741417f39ab4e7ca9bda83d438327d167f0))

## [0.5.2](https://github.com/shuttle-hq/shuttle/compare/v0.5.1..v0.5.2) - 2022-09-09

### Bug Fixes

- Helpful error when a secret is not found ([#335](https://github.com/shuttle-hq/shuttle/issues/335)) - ([757ef4d](https://github.com/shuttle-hq/shuttle/commit/757ef4d49d7597e4913a9bdbe0c1b9bc68fd3feb))
- Update meta DB state even on factory failure ([#332](https://github.com/shuttle-hq/shuttle/issues/332)) - ([94c0878](https://github.com/shuttle-hq/shuttle/commit/94c0878dbca078e721d18dc2845fba2134416ff1))

### Miscellaneous Tasks

- V0.5.2 - ([0c55a15](https://github.com/shuttle-hq/shuttle/commit/0c55a15a8918f2a3076b26781bdf2994bc3ba027))
- Do not do clean before images on pushing and no arm64 build ([#331](https://github.com/shuttle-hq/shuttle/issues/331)) - ([e755bc0](https://github.com/shuttle-hq/shuttle/commit/e755bc06523fb3ef5fe66d2a2bee37c49cd1f564))

### Miscellaneous

- Migrate the builder to x86 - ([a1ef6b7](https://github.com/shuttle-hq/shuttle/commit/a1ef6b759b012bb1bfcae783532507728730f448))
- Updating Discord Links ([#325](https://github.com/shuttle-hq/shuttle/issues/325)) - ([5576a6c](https://github.com/shuttle-hq/shuttle/commit/5576a6c434fd661ee7af7950cfb55cc443b18785))

## [0.5.1](https://github.com/shuttle-hq/shuttle/compare/v0.5.0..v0.5.1) - 2022-08-31

### Features

- Serenity integration ([#303](https://github.com/shuttle-hq/shuttle/issues/303)) - ([b333483](https://github.com/shuttle-hq/shuttle/commit/b3334838bac5861daacec3465d2531fbcc5f4298))
- Respect RUST_LOG ([#316](https://github.com/shuttle-hq/shuttle/issues/316)) - ([f37c9aa](https://github.com/shuttle-hq/shuttle/commit/f37c9aa4aa96ae4472a986fa3bf8bf7b4400980c))

### Bug Fixes

- Init rocket dependency version ([#314](https://github.com/shuttle-hq/shuttle/issues/314)) - ([af4420a](https://github.com/shuttle-hq/shuttle/commit/af4420a500d55b174d2122fc443857f2e3c248ec))

### Documentation

- Fix typos ([#278](https://github.com/shuttle-hq/shuttle/issues/278)) - ([a788b30](https://github.com/shuttle-hq/shuttle/commit/a788b30afff220f81759a0f8d24644a65c1984c3))
- Update to include new makefile info ([#315](https://github.com/shuttle-hq/shuttle/issues/315)) - ([8a4f19a](https://github.com/shuttle-hq/shuttle/commit/8a4f19af8d8d0835578e58013ed1a5cf2f343641))

### Miscellaneous Tasks

- V0.5.1 ([#322](https://github.com/shuttle-hq/shuttle/issues/322)) - ([b43536a](https://github.com/shuttle-hq/shuttle/commit/b43536a857190620a1414a60d4d0e54d865bc165))
- Dependencies upgrades ([#311](https://github.com/shuttle-hq/shuttle/issues/311)) - ([1e798c9](https://github.com/shuttle-hq/shuttle/commit/1e798c996f1c3b12f6ed96b2a3a8440142aaae49))

### Miscellaneous

- Secret local run ([#317](https://github.com/shuttle-hq/shuttle/issues/317)) - ([cfa78ef](https://github.com/shuttle-hq/shuttle/commit/cfa78ef09599aa07c0039b49bcba77c6d64a6125))
- Refactor/profanity checks ([#312](https://github.com/shuttle-hq/shuttle/issues/312)) - ([850eb2c](https://github.com/shuttle-hq/shuttle/commit/850eb2c507d15c6e902933a29891e660000aa1af))
- Placement constraints and other fixes - ([3210c6a](https://github.com/shuttle-hq/shuttle/commit/3210c6a4e01e11dfcb4e6169e79db3a02390d039))

## [0.5.0](https://github.com/shuttle-hq/shuttle/compare/v0.4.2..v0.5.0) - 2022-08-18

### Features

- Implement support for a shared MongoDB database ([#300](https://github.com/shuttle-hq/shuttle/issues/300)) - ([79df6d7](https://github.com/shuttle-hq/shuttle/commit/79df6d78d71fc5766ff30bdc66d3b6f6257bcf27))

### Bug Fixes

- Fixed title ([#309](https://github.com/shuttle-hq/shuttle/issues/309)) - ([12b5679](https://github.com/shuttle-hq/shuttle/commit/12b567927f0c0809f1006cd8866edeb6945d7add))

### Miscellaneous Tasks

- V0.5.0 ([#310](https://github.com/shuttle-hq/shuttle/issues/310)) - ([1d1ba78](https://github.com/shuttle-hq/shuttle/commit/1d1ba78c669fe9079968f05211512598b1481bab))
- Contributors [ENG-78] ([#308](https://github.com/shuttle-hq/shuttle/issues/308)) - ([6c06291](https://github.com/shuttle-hq/shuttle/commit/6c06291d6d707336eeb4a1c195b68200cf1dfe81))
- CircleCI migration ([#277](https://github.com/shuttle-hq/shuttle/issues/277)) - ([6b1a9e5](https://github.com/shuttle-hq/shuttle/commit/6b1a9e5e93244398e35386cc21dfdab39d3baf94))
- Env update ([#307](https://github.com/shuttle-hq/shuttle/issues/307)) - ([0ea14bb](https://github.com/shuttle-hq/shuttle/commit/0ea14bbc4e812d3b5bf077342e1317eb884d2e91))

### Miscellaneous

- Authentication tutorial blog post ([#301](https://github.com/shuttle-hq/shuttle/issues/301)) - ([1c212f1](https://github.com/shuttle-hq/shuttle/commit/1c212f13d77761f7e982d615c2c7a3d4565685b0))

## [0.4.2](https://github.com/shuttle-hq/shuttle/compare/v0.4.1..v0.4.2) - 2022-08-15

### Features

- Profanity filter & added "shuttle.rs" to the reserved list of project names ([#293](https://github.com/shuttle-hq/shuttle/issues/293)) - ([9f838d2](https://github.com/shuttle-hq/shuttle/commit/9f838d2f9954f8ac475081c9c14aaf7c73bdfb3b))
- Support underscore in project name ([#299](https://github.com/shuttle-hq/shuttle/issues/299)) - ([fcda8a0](https://github.com/shuttle-hq/shuttle/commit/fcda8a026fde49df058435a2cf818d023b3933a0))

### Bug Fixes

- *(blog)* Typos ([#295](https://github.com/shuttle-hq/shuttle/issues/295)) - ([5415454](https://github.com/shuttle-hq/shuttle/commit/5415454c46cecba2f98c318128227b226b30090c))
- Make profanity filter zealous, add tests to CI ([#302](https://github.com/shuttle-hq/shuttle/issues/302)) - ([52d7060](https://github.com/shuttle-hq/shuttle/commit/52d7060e789dcdc06bd208a8256cd7abf1897dba))

### Documentation

- *(examples)* Update url-shortener README ([#296](https://github.com/shuttle-hq/shuttle/issues/296)) - ([9002a38](https://github.com/shuttle-hq/shuttle/commit/9002a3833f1ce7608af7c7808ddbea1444d74d7e))

### Miscellaneous Tasks

- V0.4.2 ([#305](https://github.com/shuttle-hq/shuttle/issues/305)) - ([587a5c9](https://github.com/shuttle-hq/shuttle/commit/587a5c9b606681bcf87dc9069998f8080354ac1c))
- Env update ([#290](https://github.com/shuttle-hq/shuttle/issues/290)) - ([1ce3baa](https://github.com/shuttle-hq/shuttle/commit/1ce3baacc51e9bb937410b0dcc1ba8e3230a1edf))

### Miscellaneous

- Remove extra braces in shuttle init for axum ([#304](https://github.com/shuttle-hq/shuttle/issues/304)) - ([306789b](https://github.com/shuttle-hq/shuttle/commit/306789b7ff90bc98aa0aa150d89245f0286c174d))
- Middleware - ([1f68c11](https://github.com/shuttle-hq/shuttle/commit/1f68c11211b6e744045b85c76886d151a4f4f057))
- Copy fix ([#291](https://github.com/shuttle-hq/shuttle/issues/291)) - ([5f4fd3e](https://github.com/shuttle-hq/shuttle/commit/5f4fd3e016e250b5f9087a08dce4f486c89f9847))
- Minor grammar fixes ([#289](https://github.com/shuttle-hq/shuttle/issues/289)) - ([e993f00](https://github.com/shuttle-hq/shuttle/commit/e993f00d71078f48adf8e977cd8db32385a2f932))
- Patterns with rust types - ([467b9b5](https://github.com/shuttle-hq/shuttle/commit/467b9b5ec2eab5326ef2d3dd8d538a8f2e64e802))
- `cargo-shuttle` README ([#284](https://github.com/shuttle-hq/shuttle/issues/284)) - ([10346a1](https://github.com/shuttle-hq/shuttle/commit/10346a12494eb2f2dd200baa3688fdaff86aff15))

## [0.4.1](https://github.com/shuttle-hq/shuttle/compare/v0.4.0..v0.4.1) - 2022-07-27

### Features

- Poem-web integration ([#275](https://github.com/shuttle-hq/shuttle/issues/275)) - ([50e7cc2](https://github.com/shuttle-hq/shuttle/commit/50e7cc2e29cd5f85498b89b3c555d24327cb3376))

### Bug Fixes

- Docker-compose.dev.yml uses right image tag - ([351310c](https://github.com/shuttle-hq/shuttle/commit/351310c000fae6c8fd506c68d9b339d7c5d0bf33))

### Refactor

- Main macro to own module ([#279](https://github.com/shuttle-hq/shuttle/issues/279)) - ([5bcb78a](https://github.com/shuttle-hq/shuttle/commit/5bcb78ab1482819928f614f6dc50ae70f92ef0bf))

### Miscellaneous Tasks

- V0.4.1 ([#286](https://github.com/shuttle-hq/shuttle/issues/286)) - ([a616e72](https://github.com/shuttle-hq/shuttle/commit/a616e72c26424c691cf3cf87b2a3f85628b6e560))
- CircleCI fix ([#274](https://github.com/shuttle-hq/shuttle/issues/274)) - ([4785a54](https://github.com/shuttle-hq/shuttle/commit/4785a54ae4cda967a20425206087d76d11605b90))
- Env update ([#253](https://github.com/shuttle-hq/shuttle/issues/253)) - ([5b052ce](https://github.com/shuttle-hq/shuttle/commit/5b052ce5980887ebae20883a1f71976c02944a71))
- Update website examples for v0.4.0 ([#252](https://github.com/shuttle-hq/shuttle/issues/252)) - ([deb8b10](https://github.com/shuttle-hq/shuttle/commit/deb8b1017b45581a1c8302b7ce87b7b46bef246b))

### Miscellaneous

- *(clap)* Migrate uses of structopt to clap in api ([#256](https://github.com/shuttle-hq/shuttle/issues/256)) - ([e6a4f4f](https://github.com/shuttle-hq/shuttle/commit/e6a4f4f7559571a7cc3c5774f623e82f4b42e253))
- *(clap)* Migrate uses of structopt to clap in cargo-shuttle ([#257](https://github.com/shuttle-hq/shuttle/issues/257)) - ([ab56385](https://github.com/shuttle-hq/shuttle/commit/ab5638543c5e905446972995564464195525b165))
- Added missing symbol ([#285](https://github.com/shuttle-hq/shuttle/issues/285)) - ([3b56216](https://github.com/shuttle-hq/shuttle/commit/3b5621643f0ce81b0ee635a5faf1f9e734a85122))
- Fixes e2e Readme link ([#268](https://github.com/shuttle-hq/shuttle/issues/268)) - ([5dce6d7](https://github.com/shuttle-hq/shuttle/commit/5dce6d7aa456cfa05b410b8d8c493b9b4cd24250))
- Updates on readme for cargo shuttle init ([#264](https://github.com/shuttle-hq/shuttle/issues/264)) - ([eb20ff6](https://github.com/shuttle-hq/shuttle/commit/eb20ff65a47efd80b804711847ff0ccc66ba5bad))
- Implement `cargo shuttle init --axum|rocket|tide|tower` ([#238](https://github.com/shuttle-hq/shuttle/issues/238)) - ([d4af367](https://github.com/shuttle-hq/shuttle/commit/d4af3674de5f77477eda7febbbd7935add7f1672))
- Refactor/locate root dir ([#232](https://github.com/shuttle-hq/shuttle/issues/232)) - ([9b4d9fa](https://github.com/shuttle-hq/shuttle/commit/9b4d9fa80c896daf97aaa6675e8b303e415d6c15))

## [0.4.0](https://github.com/shuttle-hq/shuttle/compare/v0.3.3..v0.4.0) - 2022-07-11

### Features

- Docker-buildx builds and docker-compose deploys - ([9d28924](https://github.com/shuttle-hq/shuttle/commit/9d289245789fbbeeb8ac171ace71b9245d900a35))
- Improve API key error with command hint ([#217](https://github.com/shuttle-hq/shuttle/issues/217)) - ([1e2eff5](https://github.com/shuttle-hq/shuttle/commit/1e2eff5634db4ab39500b2907142181b5c079209))
- AWS RDS ([#180](https://github.com/shuttle-hq/shuttle/issues/180)) - ([6a11b03](https://github.com/shuttle-hq/shuttle/commit/6a11b03b4581eabb5c91fc5bbdeaed715930d92a))
- Locate root dir ([#203](https://github.com/shuttle-hq/shuttle/issues/203)) - ([88a0045](https://github.com/shuttle-hq/shuttle/commit/88a00457bbe08c7e1a7ee5202ea740b7f7251a69))
- Automatically set `cdylib` library type at build time ([#212](https://github.com/shuttle-hq/shuttle/issues/212)) - ([a6982d4](https://github.com/shuttle-hq/shuttle/commit/a6982d458e700d212fc4da5211d3cfa38a03024a))

### Bug Fixes

- Failing cargo shuttle deploy returns successful exit code ([#166](https://github.com/shuttle-hq/shuttle/issues/166)) - ([eea2d8f](https://github.com/shuttle-hq/shuttle/commit/eea2d8f031d47ec5753f0c2253444fa6c5e1e41b))

### Refactor

- Service ([#225](https://github.com/shuttle-hq/shuttle/issues/225)) - ([d61e628](https://github.com/shuttle-hq/shuttle/commit/d61e628418506598972d6fb08d8a187426f819ac))

### Documentation

- Add dependency to make example runnable ([#245](https://github.com/shuttle-hq/shuttle/issues/245)) - ([4629e48](https://github.com/shuttle-hq/shuttle/commit/4629e482ef4ea46afda97a0e841c353d9b935cf8))
- Relocate local setup and fix errors - ([200b3d5](https://github.com/shuttle-hq/shuttle/commit/200b3d5cdca36e9eaca80bb57751095c85a6be8c))

### Miscellaneous Tasks

- V0.4.0 ([#251](https://github.com/shuttle-hq/shuttle/issues/251)) - ([c7f5d22](https://github.com/shuttle-hq/shuttle/commit/c7f5d22745fdf6ec140e355f5592924a87c22bea))
- Docker compose ([#244](https://github.com/shuttle-hq/shuttle/issues/244)) - ([6f69de5](https://github.com/shuttle-hq/shuttle/commit/6f69de5a29a27b75d619f1fd412608a78207cacd))
- Add circleci - ([22f49a5](https://github.com/shuttle-hq/shuttle/commit/22f49a557db419daf96fad1aa20594c8d496c047))
- Add license - ([f01aa01](https://github.com/shuttle-hq/shuttle/commit/f01aa016e03c39dd58b65e078b4111cd6d47ad36))
- Clippy issues ([#239](https://github.com/shuttle-hq/shuttle/issues/239)) - ([50e7a73](https://github.com/shuttle-hq/shuttle/commit/50e7a7310b6b61a476d26546567c685ac01f4251))
- Bump automation ([#211](https://github.com/shuttle-hq/shuttle/issues/211)) - ([12a5a56](https://github.com/shuttle-hq/shuttle/commit/12a5a56a984b97c54a28181f9981bd0f5eb9a129))

### Miscellaneous

- Revert tf ([#246](https://github.com/shuttle-hq/shuttle/issues/246)) - ([043c1d2](https://github.com/shuttle-hq/shuttle/commit/043c1d258a2078839be4b2521976dde558bbe28e))
- Fix webpage link for the url-shortener example ([#243](https://github.com/shuttle-hq/shuttle/issues/243)) - ([fde6be3](https://github.com/shuttle-hq/shuttle/commit/fde6be313cbb7f5738663cd4e9c566cc76a1a404))
- Error handling - ([036f2ac](https://github.com/shuttle-hq/shuttle/commit/036f2ac9f856a789817b6194fb73fb911abb3651))
- Improved cargo shuttle version error ([#219](https://github.com/shuttle-hq/shuttle/issues/219)) - ([ed8f259](https://github.com/shuttle-hq/shuttle/commit/ed8f2591027056dbd8ccac72a67ffb3d830057e5))
- Update banner image and fixes for generative metatag images post - ([6b89a10](https://github.com/shuttle-hq/shuttle/commit/6b89a1052557f1eb1cdb07af7624a22baa27c8eb))
- Generative metatag images - ([1ed9226](https://github.com/shuttle-hq/shuttle/commit/1ed9226b7a5a8826f0a10ea01df7c95dbcdcc030))
- Add introduction to async Rust blog post - ([de444c0](https://github.com/shuttle-hq/shuttle/commit/de444c0db005992a0a5768480f4582ff71c893ed))
- Replace builder ferris - ([ada23de](https://github.com/shuttle-hq/shuttle/commit/ada23de081d75cffb19e3e810e7d6ab3d510e1b0))

## [0.3.3](https://github.com/shuttle-hq/shuttle/compare/v.0.3.1..v0.3.3) - 2022-06-10

### Features

- *(www)* Pricing page - ([7ed3e83](https://github.com/shuttle-hq/shuttle/commit/7ed3e833d714bd6fcb5b11fabf5816ab6e9289d6))
- *(www)* Add mixpanel events - ([9577c7c](https://github.com/shuttle-hq/shuttle/commit/9577c7c61d59d2e664aad86afa061848a25d58b5))
- Local run with DB ([#196](https://github.com/shuttle-hq/shuttle/issues/196)) - ([514a978](https://github.com/shuttle-hq/shuttle/commit/514a978b68674028f88b40915fe7534970aaaa29))
- Provisioner ([#199](https://github.com/shuttle-hq/shuttle/issues/199)) - ([45ac1be](https://github.com/shuttle-hq/shuttle/commit/45ac1bef207f35ca5f3cff347d230c9391f269fe))
- Check shuttle service version before deploy ([#202](https://github.com/shuttle-hq/shuttle/issues/202)) - ([6a99d6d](https://github.com/shuttle-hq/shuttle/commit/6a99d6db5133435231e4ccd3cfdb06242fe4a575))
- Example WebSocket ([#160](https://github.com/shuttle-hq/shuttle/issues/160)) - ([4467074](https://github.com/shuttle-hq/shuttle/commit/44670746671fa53e6ab7556bd96a399793772c17))
- Simple local run ([#186](https://github.com/shuttle-hq/shuttle/issues/186)) - ([57ac581](https://github.com/shuttle-hq/shuttle/commit/57ac581db63c71184eb6bd3c3ec4852f26e49fc2))

### Bug Fixes

- API crash on panic occurring in a `Service` implementation ([#168](https://github.com/shuttle-hq/shuttle/issues/168)) - ([729b008](https://github.com/shuttle-hq/shuttle/commit/729b0084e1fb9335990fcd6ae8778d572b48b0e6))

### Miscellaneous Tasks

- V0.3.3 ([#210](https://github.com/shuttle-hq/shuttle/issues/210)) - ([a21c456](https://github.com/shuttle-hq/shuttle/commit/a21c45635ccb04a07364c8d5048d80fb586ff945))
- Fix file argument - ([2f9344e](https://github.com/shuttle-hq/shuttle/commit/2f9344eeadbf25fc110d519202c85777c300b104))
- V0.3.2 ([#208](https://github.com/shuttle-hq/shuttle/issues/208)) - ([eea9acb](https://github.com/shuttle-hq/shuttle/commit/eea9acb91ebbb0ca555f2ebb2426ec10661bee6d))
- Rough edges ([#204](https://github.com/shuttle-hq/shuttle/issues/204)) - ([34d3a67](https://github.com/shuttle-hq/shuttle/commit/34d3a676591485b203ed3ff19eea61d99ce4a9aa))
- Issue 175: `cargo shuttle init` (without bonus) ([#192](https://github.com/shuttle-hq/shuttle/issues/192)) - ([6524632](https://github.com/shuttle-hq/shuttle/commit/6524632891534cc3ebe92a3690b565c496a4667a))
- Update nix pin ([#174](https://github.com/shuttle-hq/shuttle/issues/174)) - ([f359550](https://github.com/shuttle-hq/shuttle/commit/f359550c507bd16eb324648ca00142e09c5e55f4))

### Miscellaneous

- The builder pattern - ([bb9c7f2](https://github.com/shuttle-hq/shuttle/commit/bb9c7f2f40edfc5874a7dc8530174b73edbafc23))
- Add CODE_OF_CONDUCT.md and CONTRIBUTING.md ([#150](https://github.com/shuttle-hq/shuttle/issues/150)) - ([396684f](https://github.com/shuttle-hq/shuttle/commit/396684f33683168d1c11dc72d0f75a743b07979e))
- Hyper-vs-rocket - ([f81b3d6](https://github.com/shuttle-hq/shuttle/commit/f81b3d6410450975702f9f20dbafd212ef1346fe))
- #176 test on deploy ([#184](https://github.com/shuttle-hq/shuttle/issues/184)) - ([5053d3d](https://github.com/shuttle-hq/shuttle/commit/5053d3d6bb908f63a11f40a2091c48fbee294ac6))

## [.0.3.1](https://github.com/shuttle-hq/shuttle/compare/v0.3.0..v.0.3.1) - 2022-05-27

### Documentation

- Readme v2 - ([fc7e6fc](https://github.com/shuttle-hq/shuttle/commit/fc7e6fcdecb462ca26a0d5c54ff4d8c95da513f5))

### Miscellaneous Tasks

- V0.3.1 ([#191](https://github.com/shuttle-hq/shuttle/issues/191)) - ([2d725e3](https://github.com/shuttle-hq/shuttle/commit/2d725e36d8c369ea80af50132226113199a3aafd))
- Change systemd service name - ([cb91314](https://github.com/shuttle-hq/shuttle/commit/cb9131460691be822959bfd62b894fe280801abe))

## [0.3.0](https://github.com/shuttle-hq/shuttle/compare/v0.2.6..v0.3.0) - 2022-05-26

### Features

- Add type return alias ([#182](https://github.com/shuttle-hq/shuttle/issues/182)) - ([cb10833](https://github.com/shuttle-hq/shuttle/commit/cb108336f4464ff66822c912287dfa36d199e9d3))
- Tower/Hyper integration ([#159](https://github.com/shuttle-hq/shuttle/issues/159)) - ([b88e543](https://github.com/shuttle-hq/shuttle/commit/b88e54314af86f31106a86131ee8354e2bb98440))
- Runtime logs ([#158](https://github.com/shuttle-hq/shuttle/issues/158)) - ([0862f6c](https://github.com/shuttle-hq/shuttle/commit/0862f6cf4ee1943427466181ec09da2eb3c278c7))
- Implement secrets ([#144](https://github.com/shuttle-hq/shuttle/issues/144)) - ([4842f69](https://github.com/shuttle-hq/shuttle/commit/4842f691ff070d9ac9a34ada6378336759736731))

### Refactor

- Use ecr alias ([#173](https://github.com/shuttle-hq/shuttle/issues/173)) - ([5a05c0e](https://github.com/shuttle-hq/shuttle/commit/5a05c0edf2bc56bad3ee8e18c366136e3d555852))
- Conditional db ([#167](https://github.com/shuttle-hq/shuttle/issues/167)) - ([16c000a](https://github.com/shuttle-hq/shuttle/commit/16c000a3804c43ddcc56d945a6f67f870e8d8644))
- TF updates ([#183](https://github.com/shuttle-hq/shuttle/issues/183)) - ([97e09a2](https://github.com/shuttle-hq/shuttle/commit/97e09a2395ef3c205289508f7ab14dae4ab5f6ae))
- Shutdown ([#161](https://github.com/shuttle-hq/shuttle/issues/161)) - ([443de25](https://github.com/shuttle-hq/shuttle/commit/443de25d7d63c2ed765495adcbea39bcb7619ce5))

### Miscellaneous Tasks

- Bump 0.3.0 - ([e0a664d](https://github.com/shuttle-hq/shuttle/commit/e0a664d50ea2b715497908b03ad03f3f0fef505f))
- Fix formatting checks ([#162](https://github.com/shuttle-hq/shuttle/issues/162)) - ([4965edf](https://github.com/shuttle-hq/shuttle/commit/4965edf0ad8ba4f8a3d68346f2d9f083560af17c))
- Add support for --working-directory and --name parameters to `cargo shuttle status` and friends ([#122](https://github.com/shuttle-hq/shuttle/issues/122)) - ([d1522fc](https://github.com/shuttle-hq/shuttle/commit/d1522fcf64606e81f58689e75e08ce019a7ce3b6))

### Miscellaneous

- Add support to tide framework ([#172](https://github.com/shuttle-hq/shuttle/issues/172)) - ([7d46adf](https://github.com/shuttle-hq/shuttle/commit/7d46adf896d2e5edda0ec77cfe8679576ceae293))

## [0.2.6](https://github.com/shuttle-hq/shuttle/compare/v0.2.5..v0.2.6) - 2022-05-10

### Features

- Load initial user from env - ([2db73d1](https://github.com/shuttle-hq/shuttle/commit/2db73d128ee1b6193e3c56c2a0179dc3f02bfeb7))
- Allows editing proxy fqdn and api client connects to - ([6a13ff0](https://github.com/shuttle-hq/shuttle/commit/6a13ff0525517a36043ba6f0bbfa0cd0f3047559))
- Measurement logs v0.1 - ([0315e51](https://github.com/shuttle-hq/shuttle/commit/0315e512741a135a13950f95347f82d115f7b660))
- Update website (light theme and other improvements) ([#137](https://github.com/shuttle-hq/shuttle/issues/137)) - ([a1c1bd0](https://github.com/shuttle-hq/shuttle/commit/a1c1bd02313715945878038459ed873df9fa8237))
- Auth example - ([2f4759b](https://github.com/shuttle-hq/shuttle/commit/2f4759b5afec9a5aa8d45edaaeeb40948180157d))

### Bug Fixes

- Routing table start - ([c968527](https://github.com/shuttle-hq/shuttle/commit/c9685273b83af4bce92f2a99a3f71620e3e8a072))
- Devlog 1 link - ([f63f015](https://github.com/shuttle-hq/shuttle/commit/f63f015f6b7d5290b3a0b03f47e2522bc35b01ff))
- Fix file name ([#125](https://github.com/shuttle-hq/shuttle/issues/125)) - ([6c20418](https://github.com/shuttle-hq/shuttle/commit/6c204182aa0358e8a03d7730589b5fd3f246d07a))
- Fix url shortener example ([#115](https://github.com/shuttle-hq/shuttle/issues/115)) - ([c8af069](https://github.com/shuttle-hq/shuttle/commit/c8af0691a22aef6451fad425ff3631a5fe7cc757))

### Refactor

- Api_fqdn in terraform - ([8a87533](https://github.com/shuttle-hq/shuttle/commit/8a875335188677880215b611d507daff8a2ded9b))

### Documentation

- Feature flags - ([b9dec08](https://github.com/shuttle-hq/shuttle/commit/b9dec08ee21709271aa339a39f00846446004297))

### Miscellaneous Tasks

- Set ecr region to us-east-1 - ([afbadf7](https://github.com/shuttle-hq/shuttle/commit/afbadf7140f8c8f6147ba38dd5ef1c6d1321a95a))
- Testing issues - ([9a35c56](https://github.com/shuttle-hq/shuttle/commit/9a35c56221e3fc9e15da0387d9e523ba2ffa539b))
- Rename org - ([23e6c91](https://github.com/shuttle-hq/shuttle/commit/23e6c91583ce2db29f9af459176436e5404c3021))
- Public registry - ([448c62f](https://github.com/shuttle-hq/shuttle/commit/448c62f16f32fe68f5997e7c9b7bd672a656fe36))
- Bump 0.2.6 - ([fa89bd1](https://github.com/shuttle-hq/shuttle/commit/fa89bd13c254e9fef6baa2976126c3e5b025c112))
- Update to axum v0.5 - ([68a1469](https://github.com/shuttle-hq/shuttle/commit/68a1469ddcef86c95a433aa48788052687aadda4))

### Miscellaneous

- TF module - ([b7637e5](https://github.com/shuttle-hq/shuttle/commit/b7637e5686e3bacff48ac6d5bbe45781955a3fb4))
- Infrastructure from code - ([9d184a1](https://github.com/shuttle-hq/shuttle/commit/9d184a1a8a65f7ecbab790f6c4893c54e2c0ca65))
- New theme switcher ([#157](https://github.com/shuttle-hq/shuttle/issues/157)) - ([498d5d6](https://github.com/shuttle-hq/shuttle/commit/498d5d69b94eccff75b76b65e32438f62845650b))
- Devlog 1 - ([41be698](https://github.com/shuttle-hq/shuttle/commit/41be698a9ed8a5eb70ec36c4e835cb8228e5cb46))
- Update footer ([#148](https://github.com/shuttle-hq/shuttle/issues/148)) - ([7954294](https://github.com/shuttle-hq/shuttle/commit/7954294c2e18a31623ff861b9315326ed474d603))
- Fix a typo in dev log 0 - ([24760e4](https://github.com/shuttle-hq/shuttle/commit/24760e4b26e2f2e4aa63fd124ebcba470c0864d3))
- Devlog 0 - ([62d2d03](https://github.com/shuttle-hq/shuttle/commit/62d2d0334aab70b23fe75119b1a580f1e80b2fba))
- Tweak examples ([#145](https://github.com/shuttle-hq/shuttle/issues/145)) - ([29009c1](https://github.com/shuttle-hq/shuttle/commit/29009c1e5319d58ea39b240633f7a7a2f86367ad))
- Add root device (ebs) to terraform - ([277c60d](https://github.com/shuttle-hq/shuttle/commit/277c60d675a235bcda9707aada8c90335b99c442))
- Terraform systemd service - ([46586ac](https://github.com/shuttle-hq/shuttle/commit/46586ac9754a8f9af132c50145930c7e3c665886))
- Url shortener - ([544854f](https://github.com/shuttle-hq/shuttle/commit/544854fbfb9040d6f488245643cfdf3a2c5fad61))
- Remove fargate - ([c00572b](https://github.com/shuttle-hq/shuttle/commit/c00572bf27d2ca67c15fe2b83738191a1d557015))
- Blog! 🚀 ([#124](https://github.com/shuttle-hq/shuttle/issues/124)) - ([84b75c3](https://github.com/shuttle-hq/shuttle/commit/84b75c3a9d991d51389fb4ca88233894ce1a8cfb))
- Updated home page ([#102](https://github.com/shuttle-hq/shuttle/issues/102)) - ([a94e3d5](https://github.com/shuttle-hq/shuttle/commit/a94e3d52ccb975164f6fc7c01725a5ccba358863))

## [0.2.5](https://github.com/shuttle-hq/shuttle/compare/v0.2.4..v0.2.5) - 2022-03-31

### Features

- Support axum - ([eb308b5](https://github.com/shuttle-hq/shuttle/commit/eb308b59bce8c29172a6e599bbb6dfb48f9606bb))
- Generate the entrypoint using a proc_macro - ([6790156](https://github.com/shuttle-hq/shuttle/commit/67901562bfdd5d9ac48fe3b21b72a09b62c6f92b))
- Cap maximum deploys for an api instance - ([e83f8e3](https://github.com/shuttle-hq/shuttle/commit/e83f8e3a4b8d16e68d1a451a7518893dc5c01edc))
- Use github connection by default ([#90](https://github.com/shuttle-hq/shuttle/issues/90)) - ([194383d](https://github.com/shuttle-hq/shuttle/commit/194383d24cd6b40bd619d0992388015c60f7ce4d))
- Restore get_postgres_connection_pool - ([6682637](https://github.com/shuttle-hq/shuttle/commit/6682637c708de952fc5d60600db88b87fb9ef1eb))
- Set server response header so shuttle - ([45cbde9](https://github.com/shuttle-hq/shuttle/commit/45cbde91632b87db11c29908ec63d2ddddd7ffc0))

### Bug Fixes

- Pin syn, quote and proc_macro2 and enable syn/full - ([6f0763e](https://github.com/shuttle-hq/shuttle/commit/6f0763efcd78b681e754afd1000765afe520c000))
- Fix hero code copy ([#110](https://github.com/shuttle-hq/shuttle/issues/110)) - ([fe2b5f1](https://github.com/shuttle-hq/shuttle/commit/fe2b5f16d076fbac51b3a31ee91c9f4a55bb2e83))
- Wait for pg before starting api - ([31b41eb](https://github.com/shuttle-hq/shuttle/commit/31b41eb13b6caf7ce4d2d4988e2084bdc1b7a145))
- Lock users api and allow re-issuing keys - ([acc23cb](https://github.com/shuttle-hq/shuttle/commit/acc23cb61ed87839044d3dc8046b6f8ba5d581b1))
- Uri of deployed DB is now showing - ([5e35d61](https://github.com/shuttle-hq/shuttle/commit/5e35d614c85e0003589dbb12a657954a31b42878))

### Refactor

- *(api,common,cargo-shuttle)* Remove ProjectConfig and others - ([934b99f](https://github.com/shuttle-hq/shuttle/commit/934b99f4f271c460fa356f471d893d8304b5619d))
- Loader ([#101](https://github.com/shuttle-hq/shuttle/issues/101)) - ([af73643](https://github.com/shuttle-hq/shuttle/commit/af736438b0eae01e133806028d94be9abbfd3247))
- Use mpsc channel for job queue - ([4315e34](https://github.com/shuttle-hq/shuttle/commit/4315e34a0d792a2c1a97096e5da0323fea384446))
- Make sleep in client async - ([afab9b3](https://github.com/shuttle-hq/shuttle/commit/afab9b34ca4ee6882998bb814a946de12c8b252d))

### Testing

- Cleanup after run - ([70d86d4](https://github.com/shuttle-hq/shuttle/commit/70d86d4e482ca3efcb8de71812f135ca76e01266))

### Miscellaneous Tasks

- Bump 0.2.5 - ([097ca72](https://github.com/shuttle-hq/shuttle/commit/097ca72ebebf3027a163e02142877e7760192471))
- Clippy and fmt - ([7c6101b](https://github.com/shuttle-hq/shuttle/commit/7c6101be540c073ce1da406ce5578a51d6634cb8))
- Clippy and rustfmt checks - ([a4f3b22](https://github.com/shuttle-hq/shuttle/commit/a4f3b22c9a0edcd1e318f1aac9669ad4aa378cc9))
- Remove unneeded async_trait markers - ([4f6722e](https://github.com/shuttle-hq/shuttle/commit/4f6722e4a823f03ab2379c88af54ef22aff5872e))
- Only deploy on release, api uses release cargo-service - ([d13631c](https://github.com/shuttle-hq/shuttle/commit/d13631cb381187091de3386acb2973c4199d3f3c))

### Miscellaneous

- Implement url shortener with postgres and rocket ([#94](https://github.com/shuttle-hq/shuttle/issues/94)) - ([9ea2407](https://github.com/shuttle-hq/shuttle/commit/9ea2407d837e855f971c584aa622d49ddf29dbc3))
- Don't try to remove .so files ([#109](https://github.com/shuttle-hq/shuttle/issues/109)) - ([0ac83bf](https://github.com/shuttle-hq/shuttle/commit/0ac83bf99d5a8822995c9c5527579cc084bd0002))

## [0.2.4] - 2022-03-17

### Features

- Login flow on website ([#67](https://github.com/shuttle-hq/shuttle/issues/67)) - ([89de909](https://github.com/shuttle-hq/shuttle/commit/89de909dd4d1ed16fafa112cd405923bf53f29f8))
- Cargo auth - ([672e1b8](https://github.com/shuttle-hq/shuttle/commit/672e1b81eb169a99c630eda05cb35b961f1d2b4b))
- Shuttle.rs 🚀 ([#45](https://github.com/shuttle-hq/shuttle/issues/45)) - ([aeb8508](https://github.com/shuttle-hq/shuttle/commit/aeb85084fdc85c773583801749287ccc534b808e))
- Postgres example ([#31](https://github.com/shuttle-hq/shuttle/issues/31)) - ([bba246d](https://github.com/shuttle-hq/shuttle/commit/bba246d5e2364464e14b60e9381d55563ffbdb3f))
- Api endpoint - ([5a13669](https://github.com/shuttle-hq/shuttle/commit/5a13669130d9945e56b0fcdf660f73d002b0d384))
- Add deployment error messages - ([f38e77a](https://github.com/shuttle-hq/shuttle/commit/f38e77a0794449e981d2092e31f0db5d4cdb8570))
- Improve client errors - ([5364c04](https://github.com/shuttle-hq/shuttle/commit/5364c04633e372b66e2058d73a1a444a710df91a))
- Project name validation - ([dabb6c4](https://github.com/shuttle-hq/shuttle/commit/dabb6c48451bcf0dcdd3811497ad403116ee3dc6))
- Database deployment - ([aa1170d](https://github.com/shuttle-hq/shuttle/commit/aa1170ddc543200680cc7c7001292a1319b26dcf))
- Parameterised user toml ([#38](https://github.com/shuttle-hq/shuttle/issues/38)) - ([c226852](https://github.com/shuttle-hq/shuttle/commit/c22685210f8d709dcbc7f408323a0dd43a8d2251))
- Initialise deployment service from state - ([0d1e25a](https://github.com/shuttle-hq/shuttle/commit/0d1e25a7986bb644ec806372591b560e85b9652d))
- Project to user mapping - ([2c92d03](https://github.com/shuttle-hq/shuttle/commit/2c92d03869c8d0b112ac26f0363c697eb834777b))
- Naive api keys - ([8698ef0](https://github.com/shuttle-hq/shuttle/commit/8698ef027b62e3a79b41d1e4197f4ae823588dd3))
- Do not require unveil toml - ([3de48fb](https://github.com/shuttle-hq/shuttle/commit/3de48fb0b70c7ebc82dbc72b5d66373736ca6429))
- Factory trait - ([70ef43e](https://github.com/shuttle-hq/shuttle/commit/70ef43eafb4e51d4a57f0f972d0895351eaa8a23))
- Status and delete command - ([7c5884e](https://github.com/shuttle-hq/shuttle/commit/7c5884e089ac44e458264e7e6b0948586109cc30))
- Allow dirty & cargo-unveil uses structopt - ([4dd035f](https://github.com/shuttle-hq/shuttle/commit/4dd035fd5c27838147ef9a0db16cac0473aadef0))
- Delete deployment rebased - ([cc510c3](https://github.com/shuttle-hq/shuttle/commit/cc510c3426280ed2bed556b6f7d9fcb8c8826f93))
- Host-based routing - ([d7834c4](https://github.com/shuttle-hq/shuttle/commit/d7834c4ebdfdc906ded39c62c6d3ca063a1490a0))
- Capture build output - ([e20269a](https://github.com/shuttle-hq/shuttle/commit/e20269a005555c64575db5908ad7a0eb8b868101))
- Get so path - ([02b911a](https://github.com/shuttle-hq/shuttle/commit/02b911aea747459062665ecd556449d0b283ac0e))
- Implement deployment of Rocket application on assigned port - ([a3a362f](https://github.com/shuttle-hq/shuttle/commit/a3a362fe94e10433bd4a51b2bb2cac23070911b2))
- Use project config for deployment - ([6258661](https://github.com/shuttle-hq/shuttle/commit/6258661ba738b1e5711b3292ac49a03a79a1e884))
- Specify crates folder - ([41ed5c7](https://github.com/shuttle-hq/shuttle/commit/41ed5c7743e69ffb51ff0353f574be102ca7de70))
- Global config is not toml instead of json - ([c166df7](https://github.com/shuttle-hq/shuttle/commit/c166df7f5cfc64692fc47df99f3aa15cd0c54f5f))
- Deployment job processor - ([79ee12d](https://github.com/shuttle-hq/shuttle/commit/79ee12de77983ea78c6dccbc57e3e8974ae78696))
- Load cdylib library and import user's implementation of `Service` ([#2](https://github.com/shuttle-hq/shuttle/issues/2)) - ([aafa39f](https://github.com/shuttle-hq/shuttle/commit/aafa39f7a1f8475d665d5ff404fd703583c23271))

### Bug Fixes

- Discord url ([#79](https://github.com/shuttle-hq/shuttle/issues/79)) - ([a3e323a](https://github.com/shuttle-hq/shuttle/commit/a3e323ae9fd903d2e557c37eb5e2843e79911eaa))
- Api key modal issues ([#78](https://github.com/shuttle-hq/shuttle/issues/78)) - ([44b838d](https://github.com/shuttle-hq/shuttle/commit/44b838d9a7ca9c21cbbf69479381693a1cd3d318))
- Cargo build poisons lock ([#66](https://github.com/shuttle-hq/shuttle/issues/66)) - ([5e1ec9d](https://github.com/shuttle-hq/shuttle/commit/5e1ec9d53ce4989feffc03262a55189a28dbcb3e))
- Api now dies if it cannot find the users toml - ([74de9ab](https://github.com/shuttle-hq/shuttle/commit/74de9abc0c00c86c6cc8ca2f90d24c24f781d0cd))
- Www ci (it's main not master) ([#60](https://github.com/shuttle-hq/shuttle/issues/60)) - ([8d187b2](https://github.com/shuttle-hq/shuttle/commit/8d187b289215667e3070c3209450c295dc1d6689))
- Lowercase shuttle ([#59](https://github.com/shuttle-hq/shuttle/issues/59)) - ([19333e3](https://github.com/shuttle-hq/shuttle/commit/19333e3872862bd263f58b2c3ac6c766499fb06f))
- New svc not deployed - ([999a879](https://github.com/shuttle-hq/shuttle/commit/999a8790e129681ecd021f1524d8055b2e99d858))
- Fs root cfg release - ([e262737](https://github.com/shuttle-hq/shuttle/commit/e262737082333c6bfec69c4ca75ae6585741c0a5))
- Deleted state on delete - ([4e24ea0](https://github.com/shuttle-hq/shuttle/commit/4e24ea012421caaf6a3e33af68cbbf2d68f2f9a3))
- Persistence regression - ([ef8e117](https://github.com/shuttle-hq/shuttle/commit/ef8e11709972a245364638271951b3a6a45b9238))
- Use `log::debug!` over `dbg!` - ([72c7f58](https://github.com/shuttle-hq/shuttle/commit/72c7f585aa987b710e0c1801db938813f6942034))

### Refactor

- Make it easier to implement factory - ([6d88115](https://github.com/shuttle-hq/shuttle/commit/6d88115ee201e3d3df2d4bf8e0ae0f745cb02b9d))
- Make deployed services have their own runtime - ([6c6e88b](https://github.com/shuttle-hq/shuttle/commit/6c6e88bf51374666abee10cbbb63ce1f09eade06))
- Have client retry on failures - ([e7448ca](https://github.com/shuttle-hq/shuttle/commit/e7448ca749d95afe486e2fdafa66363410265897))
- Enum variants should be camel case + document `DeploymentState` variants - ([ae50b10](https://github.com/shuttle-hq/shuttle/commit/ae50b1030eb76543a10ebd4e2090dd9654a91b23))
- Make API async - ([42a87a4](https://github.com/shuttle-hq/shuttle/commit/42a87a4c2df032df67f900bb9980709e982a2d91))

### Documentation

- Update service docs login - ([6113176](https://github.com/shuttle-hq/shuttle/commit/611317602e9af487d51dc96ebb9ade25106847b8))
- Add readme ([#64](https://github.com/shuttle-hq/shuttle/issues/64)) - ([4661356](https://github.com/shuttle-hq/shuttle/commit/4661356c8c4088425ec7d7f84e81395c6b74fb28))
- Initial commit - ([a9a5e0b](https://github.com/shuttle-hq/shuttle/commit/a9a5e0b2ce800dcde2dfe9486b77ab6cb81c5830))

### Miscellaneous Tasks

- Fix website workflow env ([#77](https://github.com/shuttle-hq/shuttle/issues/77)) - ([bf62717](https://github.com/shuttle-hq/shuttle/commit/bf62717031c2af67020d9a58f262a173d4f82cf7))
- Fix website workflow ([#76](https://github.com/shuttle-hq/shuttle/issues/76)) - ([73a1156](https://github.com/shuttle-hq/shuttle/commit/73a1156fa62616eb8c86525bccafcb8952bdb117))
- Cargo-shuttle@0.2.3 - ([6b38fb3](https://github.com/shuttle-hq/shuttle/commit/6b38fb3b87779fc3d6683c87f61f8725693df187))
- Rename to shuttle - ([66190fa](https://github.com/shuttle-hq/shuttle/commit/66190fa1c6d63e9b44a10c3d2d9613a7119b56c2))
- Deploy at the end of action - ([08938a1](https://github.com/shuttle-hq/shuttle/commit/08938a1a0be10584ffa2885004f71666d81f6719))
- Bump service 0.2.0 - ([b7950a7](https://github.com/shuttle-hq/shuttle/commit/b7950a7eaaca84d9740ed536f70acc1d9312e2f2))
- Add postgres to Dockerfile and deployment - ([b74d6c8](https://github.com/shuttle-hq/shuttle/commit/b74d6c8d00b6647f70c7b100900e9ab899c4ea8b))
- Added rustfmt.toml and formatted repo - ([b630074](https://github.com/shuttle-hq/shuttle/commit/b630074d19ba307aa7a44082c3cc7400d73331c4))
- Simple e2e for hello world - ([6acff1a](https://github.com/shuttle-hq/shuttle/commit/6acff1aff622ed00f9c45b03a8796fd8b55537a7))
- Service on crates.io - ([c0e98ef](https://github.com/shuttle-hq/shuttle/commit/c0e98ef2b6a0455ef14a0b876ecf86d38ae6ee2e))

### Miscellaneous

- Bump AMI - ([ffdf198](https://github.com/shuttle-hq/shuttle/commit/ffdf1983b94a56e30688651b647aa57ca7fbb8bc))
- Users.toml - ([b8c362a](https://github.com/shuttle-hq/shuttle/commit/b8c362a915828d906706f4472606ebc368d263b5))
- Use self-hosted runners ([#54](https://github.com/shuttle-hq/shuttle/issues/54)) - ([6cecc29](https://github.com/shuttle-hq/shuttle/commit/6cecc29b321e0608d869a405e1e85a5ff8d7ebe7))
- Set up domains and add tls - ([45b7c17](https://github.com/shuttle-hq/shuttle/commit/45b7c1771688d3c9aef333e97d4278b7ac0f981c))
- Change name of lbs - ([d996d3a](https://github.com/shuttle-hq/shuttle/commit/d996d3a85b58d32598c453d6e4702b4d7c73cb4a))
- Getting started - ([405eaed](https://github.com/shuttle-hq/shuttle/commit/405eaedf0d51c24e59af4f8cc927f251126f1419))
- Added workspace - ([7459845](https://github.com/shuttle-hq/shuttle/commit/745984586bd5791e6cf38f4946c5ee497c0820df))
- Initial commit - ([83771fb](https://github.com/shuttle-hq/shuttle/commit/83771fb10cb39cc63584f459e2c8958a5e6ffeda))

<!-- generated by git-cliff -->
