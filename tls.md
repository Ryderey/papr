• ## 1. 根因

  这个报错不是“GitHub 证书过期”，而是：

  > Tauri 下载器无法把服务器证书链连接到一个受信任的根 CA。

  当前项目使用 @tauri-apps/cli 2.11.1。该版本下载 WiX 时使用 Rust TLS，并启用了平台证书验证；Windows 下会调用 Windows
  证书存储和验证 API。因此这次错误优先指向 Windows 信任链或 HTTPS 代理，而不是 pnpm 配置。Tauri 2.11.1 源码
  (https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.11.1/crates/tauri-bundler/Cargo.toml)、rustls-platform-verifier
  说明 (https://github.com/rustls/rustls-platform-verifier)

  最常见原因依次是：

  1. 公司代理、防火墙、VPN 或杀毒软件进行了 HTTPS 解密，重新签发了 GitHub 证书，但企业根 CA 没有正确安装。
  2. 企业 CA 只安装在 CurrentUser，没有安装到 LocalMachine，导致服务、CI 或某些原生工具不信任。
  3. Windows 自动根证书更新被组策略关闭，或机器长期无法访问 Windows Update。
  4. 系统时间严重错误。
  5. 某些工具使用自己的 CA 文件，而不是 Windows 证书库。严格来说，不存在所有开发工具共享的“唯一系统 TLS 证书库”。

  ## 2. 处理方法

  ### 第一步：判断是系统问题还是企业代理

  在 PowerShell 中执行：

  Get-Date
  w32tm /query /status

  curl.exe -I https://github.com
  curl.exe -I https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip

  Invoke-WebRequest https://github.com -Method Head
  netsh winhttp show proxy

  检查代理环境变量是否存在，但不要输出其中可能包含的账号密码：

  'HTTP_PROXY','HTTPS_PROXY','ALL_PROXY','NO_PROXY' | ForEach-Object {
      [PSCustomObject]@{
          Name = $_
          Set  = $null -ne (Get-Item "Env:$_" -ErrorAction SilentlyContinue)
      }
  }

  然后在浏览器打开 https://github.com，查看证书的“颁发者”：

  - 如果是 DigiCert 等公开 CA：重点检查 Windows 根证书更新。
  - 如果是公司名、Zscaler、Fortinet、ESET、Kaspersky 等：已经确认存在 HTTPS 拦截，必须部署对应企业根 CA。
  - 如果浏览器正常但 curl.exe 和 Tauri 都失败：通常是证书只装到了浏览器私有库，或证书链部署不完整。

  ### 第二步：修复 Windows 根证书更新

  检查相关服务与策略：

  Get-Service CryptSvc,wuauserv,bits

  reg query "HKLM\SOFTWARE\Policies\Microsoft\SystemCertificates\AuthRoot" /v DisableRootAutoUpdate

  判断方式：

  - 键不存在或值为 0：通常允许自动更新。
  - 值为 1：自动根证书更新被关闭，需要修改域策略，而不是在单台机器上反复手工修复。
  - 服务被禁用：恢复 Windows Update、Cryptographic Services 和 BITS 的正常策略。

  联网电脑应通过 Windows Update 接收可信 CTL；微软说明可直接连接 Windows Update
  的电脑会定期获得更新。隔离网络应由管理员使用 certutil -syncWithWU 同步 CTL，再通过内网和组策略分发。微软可信根证书配置
  (https://learn.microsoft.com/en-us/windows-server/identity/ad-cs/configure-trusted-roots-disallowed-certificates)

  普通开发机建议：

  1. 打开 Windows Update。
  2. 安装累计更新和安全更新。
  3. 确认时间、时区和时间同步正常。
  4. 重启 Windows。
  5. 不要从第三方网站下载所谓“完整根证书包”。

  ### 第三步：正确部署企业根 CA

  向网络或安全管理员索取：

  - 企业根 CA 证书。
  - 必要的中间 CA 证书。
  - 官方 SHA-256 指纹。

  必须通过独立渠道核对指纹：

  Get-FileHash C:\Certificates\corp-root.cer -Algorithm SHA256

  以管理员身份安装：

  Import-Certificate `
    -FilePath C:\Certificates\corp-root.cer `
    -CertStoreLocation Cert:\LocalMachine\Root

  中间证书应装到 CA，不能混进根证书库：

  Import-Certificate `
    -FilePath C:\Certificates\corp-intermediate.cer `
    -CertStoreLocation Cert:\LocalMachine\CA

  微软官方 Import-Certificate 文档确认了 LocalMachine\Root 的用法。Import-Certificate
  (https://learn.microsoft.com/en-us/powershell/module/pki/import-certificate)

  企业环境中更彻底的方式是通过域 GPO 或 MDM 将根 CA 部署到：

  Computer Configuration
    → Policies
    → Windows Settings
    → Security Settings
    → Public Key Policies
    → Trusted Root Certification Authorities

  不要：

  - 导入 GitHub 的站点叶子证书。
  - 导入来源不明的根证书。
  - 只安装到浏览器。
  - 每台电脑手工维护企业 CA。

  安装完成后关闭并重新打开终端、IDE，然后重新执行：

  pnpm tauri build

  ### 第四步：统一各开发工具的信任策略

  系统证书修好后，仍需处理不使用 Windows 证书库的工具。

  Node.js 22.19+、24.6+ 可以显式启用系统 CA：

  $env:NODE_USE_SYSTEM_CA = '1'
  pnpm install

  较旧 Node.js 可使用组织提供的 PEM：

  $env:NODE_EXTRA_CA_CERTS = 'C:\Certificates\corp-chain.pem'

  Node 官方说明 NODE_USE_SYSTEM_CA=1 会将系统证书库与 Node 自带 CA 一起使用。Node.js CLI 文档
  (https://nodejs.org/api/cli.html#node_use_system_ca1)

  Git for Windows 可测试 Schannel：

  git -c http.sslBackend=schannel ls-remote https://github.com/wixtoolset/wix3.git

  确认有效后，才考虑持久化：

  git config --global http.sslBackend schannel

  Git 官方说明 schannel 使用 Windows TLS 后端；还应检查是否存在过期的自定义 http.sslCAInfo 配置。Git TLS 配置
  (https://git-scm.com/docs/git-config#Documentation/git-config.txt-httpsslBackend)

  另外，当前项目的 src-tauri/Cargo.toml:42 明确使用：

  rustls-tls-webpki-roots

  这意味着应用自身的 reqwest 请求使用内置 WebPKI 根证书，不会自动信任企业本地根 CA。这不影响本次 Tauri CLI 下载 WiX，但
  可能导致应用运行后的网络请求继续报 UnknownIssuer。如果应用必须适配企业 HTTPS 代理，应评估改为系统原生根证书功能，或显
  式加载经过验证的企业 CA。

  ### 严禁使用的“解决方案”

  以下配置只是关闭安全验证：

  NODE_TLS_REJECT_UNAUTHORIZED=0
  npm/pnpm strict-ssl=false
  GIT_SSL_NO_VERIFY=true
  git config http.sslVerify false
  danger_accept_invalid_certs

  这些设置会让代理、中间人攻击和伪造下载包不再可检测，不应进入开发机、CI 或项目配置。

  ## 3. 修改文件

  未修改项目文件。

  ## 4. 验证结果

  已确认：

  - 项目使用 Tauri CLI 2.11.1。
  - 该版本下载并校验 WiX 3.14 压缩包。
  - 下载器默认启用 Rust TLS 和 Windows 平台证书验证。
  - 项目自身的 reqwest 使用独立 WebPKI 根证书。

  尚未直接执行当前机器的 TLS 和证书库诊断，因此还不能断言具体是企业代理、根证书更新还是系统时间问题。

  ## 5. 风险和后续建议

  最彻底的企业级方案是：

  1. Windows 自动根证书更新保持启用。
  2. 企业 HTTPS 解密根 CA 通过 GPO/MDM 部署到 LocalMachine\Root。
  3. 中间 CA 部署到 LocalMachine\CA。
  4. Node、Git、Rust、Java、Python 明确采用系统证书库或统一的受管 CA 包。
  5. CI 镜像同样部署 CA，而不是关闭 TLS 验证。
  6. 建立定期检查，提前监控企业 CA 的到期时间。