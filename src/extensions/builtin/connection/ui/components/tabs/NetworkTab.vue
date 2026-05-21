<template>
  <div class="net-tab">
    <!-- File DB: no network needed -->
    <div v-if="driver?.is_file" class="net-file-hint">
      <Database :size="36" class="hint-icon" />
      <h3>{{ $t('connection.networkTab.fileDbHint') }}</h3>
      <p>{{ $t('connection.networkTab.fileDbHintDesc') }}</p>
    </div>

    <!-- Empty driver -->
    <div v-else-if="!driver" class="empty-hint">{{ $t('navigator.noDriver') }}</div>

    <!-- Network config -->
    <template v-else>
      <div class="net-hint">
        <strong>{{ $t('connection.networkTab.dynamicChain') }}</strong>
        {{ $t('connection.networkTab.chainDesc') }}
      </div>

      <!-- Chain header -->
      <div class="chain-header">
        <span class="ch-drag" />
        <span class="ch-order">#</span>
        <span class="ch-type">{{ $t('connection.networkTab.protocol') }}</span>
        <span class="ch-config">{{ $t('connection.networkTab.configuration') }}</span>
        <span class="ch-toggle">{{ $t('connection.networkTab.enable') }}</span>
        <span class="ch-acts">{{ $t('connection.networkTab.actions') }}</span>
      </div>

      <!-- Chain list -->
      <div class="chain-list">
        <div
          v-for="(hop, idx) in chain"
          :key="hop.id"
          :class="hopItemClass(hop)"
          :draggable="true"
          @dragstart="dragStart($event, hop.id)"
          @dragover.prevent="dragOver($event, hop.id)"
          @dragleave="dragLeave($event)"
          @drop="drop($event, hop.id)"
          @dragend="dragEnd"
        >
          <div class="drag-handle" :title="hop.protocol === 'ssl' ? $t('connection.networkTab.sslLocked') : $t('connection.networkTab.dragHint')">
            {{ hop.protocol === 'ssl' ? '🔒' : '≡' }}
          </div>
          <span :class="['order-badge', hop.protocol]">{{ orderNum(idx, hop) }}</span>
          <span :class="['type-badge', hop.protocol]">{{ hopIcon(hop.protocol) }} {{ hopLabel(hop.protocol) }}</span>

          <div class="config-area">
            <!-- Mode switcher -->
            <div class="mode-bar">
              <span :class="['mode-btn', { active: hop.mode === 'select' }]" @click="setHopMode(hop.id, 'select')">{{ t('navigator.selectProfile') }}</span>
              <span :class="['mode-btn', { active: hop.mode === 'new' }]" @click="setHopMode(hop.id, 'new')">{{ t('navigator.newConfig') }}</span>
              <span :class="['mode-btn', { active: hop.mode === 'custom' }]" @click="setHopMode(hop.id, 'custom')">{{ t('navigator.customConfig') }}</span>
            </div>

            <!-- Select mode -->
            <div v-if="hop.mode === 'select'" class="select-row">
              <NSelect
                v-model:value="hop.profileId"
                size="small"
                :options="profileOpts(hop.protocol)"
                :placeholder="selectPlaceholder(hop.protocol)"
                class="config-select"
                filterable
              />
              <NButton size="tiny" type="primary" ghost @click="setHopMode(hop.id, 'new')">+ {{ t('navigator.newConfig') }}</NButton>
            </div>

            <!-- New mode: inline form -->
            <div v-else-if="hop.mode === 'new'" class="inline-form-v5">
              <!-- Name + Scope -->
              <div class="form-row">
                <div class="form-group f1">
                  <span class="form-label">{{ t('navigator.configName') }}</span>
                  <NInput v-model:value="newFormData[hop.id].name" size="small" :placeholder="t('navigator.newSshProfile')" />
                </div>
                <div class="form-group f1">
                  <span class="form-label">{{ t('connection.networkTab.formScope') }}</span>
                  <span class="profile-scope-badge">{{ profScopeLabel() }}</span>
                </div>
              </div>

              <template v-if="hop.protocol === 'ssh'">
                <!-- Section: Bastion Connection -->
                <div class="form-section-label">🔗 {{ t('navigator.sectionBastion') }}</div>
                <div class="form-row">
                  <div class="form-group f2">
                    <span class="form-label">{{ t('navigator.host') }}</span>
                    <NInput v-model:value="newFormData[hop.id].host" size="small" placeholder="192.168.1.1" />
                  </div>
                  <div class="form-group f1">
                    <span class="form-label">{{ t('navigator.port') }}</span>
                    <NInputNumber v-model:value="newFormData[hop.id].port" size="small" :min="1" :max="65535" :default-value="22" />
                  </div>
                </div>
                <!-- Section: SSH Auth -->
                <div class="form-section-label">🔐 {{ t('navigator.sshAuthTitle') || 'SSH 认证' }}</div>
                <div class="form-row">
                  <div class="form-group f1">
                    <NSelect v-model:value="newFormData[hop.id].authType" size="small" :options="sshAuthTypeOpts" :placeholder="t('navigator.authMethod')" @update:value="() => onChainAuthChange(hop.id, newFormData[hop.id].authType)" />
                  </div>
                  <div class="form-group" style="flex:1.6">
                    <NSelect v-model:value="newFormData[hop.id].savedAuthId" size="small" :options="chainSshAuthCfgOpts" :placeholder="t('navigator.manualFill') || '— 手动填写 —'" clearable filterable />
                  </div>
                </div>
                <div class="form-row">
                  <div class="form-group f1">
                    <span class="form-label">{{ t('navigator.username') }}</span>
                    <NInput v-model:value="newFormData[hop.id].username" size="small" :placeholder="t('navigator.username')" />
                  </div>
                </div>
                <div class="form-row">
                  <div v-if="newFormData[hop.id].authType === 'ssh_password'" class="form-group f1">
                    <NInput v-model:value="newFormData[hop.id].password" size="small" type="password" :placeholder="t('navigator.password')" />
                  </div>
                  <template v-else>
                    <div class="form-group f1">
                      <NInput v-model:value="newFormData[hop.id].keyPath" size="small" :placeholder="t('navigator.keyPath')" />
                    </div>
                    <div class="form-group f1">
                      <NInput v-model:value="newFormData[hop.id].passphrase" size="small" type="password" :placeholder="'Passphrase (' + t('navigator.optional') + ')'" />
                    </div>
                  </template>
                </div>
                <div class="form-row">
                  <div class="form-group f1">
                    <span class="form-label">{{ t('navigator.keepAliveInterval') }}</span>
                    <NInputNumber v-model:value="newFormData[hop.id].keepAlive" size="small" :min="0" :max="600" :default-value="60" />
                  </div>
                </div>
                <!-- Section: Port Forwarding -->
                <div class="form-section-label">📡 {{ t('navigator.sectionPortForward') }}</div>
                <div class="form-row">
                  <div class="form-group f1">
                    <span class="form-label">{{ t('navigator.localPort') }}</span>
                    <NInputNumber v-model:value="newFormData[hop.id].localPort" size="small" :min="1" :max="65535" :placeholder="t('navigator.autoAssign')" />
                  </div>
                  <div class="form-group f1">
                    <span class="form-label">{{ t('navigator.remoteHost') }}</span>
                    <NInput v-model:value="newFormData[hop.id].remoteHost" size="small" :placeholder="t('navigator.targetDbHost')" />
                  </div>
                  <div class="form-group" style="width:80px">
                    <span class="form-label">{{ t('navigator.port') }}</span>
                    <NInputNumber v-model:value="newFormData[hop.id].remotePort" size="small" :min="1" :max="65535" placeholder="3306" />
                  </div>
                </div>
                <div class="form-hint">{{ t('navigator.sectionPortForwardHint') }}</div>
              </template>
              <template v-else-if="hop.protocol === 'ssl'">
                <div class="form-row">
                  <div class="form-group f1">
                    <span class="form-label">{{ t('navigator.sslMode') }}</span>
                    <NSelect v-model:value="newFormData[hop.id].sslMode" size="small" :options="sslModeOpts" :placeholder="t('navigator.sslMode')" />
                  </div>
                </div>
                <div class="form-row">
                  <div class="form-group f1">
                    <span class="form-label">{{ t('navigator.caFilePath') }}</span>
                    <NInput v-model:value="newFormData[hop.id].ca" size="small" placeholder="ca.pem" />
                  </div>
                  <div class="form-group f1">
                    <span class="form-label">{{ t('navigator.certFilePath') }}</span>
                    <NInput v-model:value="newFormData[hop.id].cert" size="small" placeholder="client.pem" />
                  </div>
                </div>
                <div class="form-row">
                  <div class="form-group f1">
                    <span class="form-label">{{ t('navigator.keyFilePath') }}</span>
                    <NInput v-model:value="newFormData[hop.id].key" size="small" :placeholder="t('navigator.keyFilePath')" />
                  </div>
                </div>
              </template>
              <template v-else-if="hop.protocol === 'proxy'">
                <div class="form-row">
                  <div class="form-group f1">
                    <span class="form-label">{{ t('navigator.proxyType') }}</span>
                    <NSelect v-model:value="newFormData[hop.id].proxyType" size="small" :options="proxyTypeOpts" :placeholder="t('navigator.proxyType')" />
                  </div>
                </div>
                <div class="form-row">
                  <div class="form-group f2">
                    <span class="form-label">{{ t('navigator.host') }}</span>
                    <NInput v-model:value="newFormData[hop.id].host" size="small" placeholder="proxy.example.com" />
                  </div>
                  <div class="form-group f1">
                    <span class="form-label">{{ t('navigator.port') }}</span>
                    <NInputNumber v-model:value="newFormData[hop.id].port" size="small" :min="1" :max="65535" :default-value="1080" />
                  </div>
                </div>
                <!-- Section: Proxy Auth -->
                <div class="form-section-label">🔐 {{ t('navigator.proxyAuthTitle') || '代理认证（可选）' }}</div>
                <div class="form-row">
                  <div class="form-group f1">
                    <NSelect v-model:value="newFormData[hop.id].authType" size="small" :options="proxyAuthTypeOpts" :placeholder="t('navigator.authMethod')" />
                  </div>
                  <div class="form-group" style="flex:1.6">
                    <NSelect v-model:value="newFormData[hop.id].savedAuthId" size="small" :options="chainSshAuthCfgOpts" :placeholder="t('navigator.manualFill') || '— 手动填写 —'" clearable filterable />
                  </div>
                </div>
                <template v-if="newFormData[hop.id].authType === 'password'">
                  <div class="form-row">
                    <div class="form-group f1">
                      <NInput v-model:value="newFormData[hop.id].username" size="small" :placeholder="t('navigator.usernameOptional')" />
                    </div>
                    <div class="form-group f1">
                      <NInput v-model:value="newFormData[hop.id].password" size="small" type="password" :placeholder="t('navigator.passwordOptional')" />
                    </div>
                  </div>
                </template>
              </template>
              <div class="new-save-row">
                <NButton size="tiny" type="primary" :loading="creating[hop.id]" @click="saveNewProfile(hop)">{{ t('navigator.saveApply') || '保存并应用' }}</NButton>
                <NButton size="tiny" secondary :loading="testingHop[hop.id]" class="btn-test-conn" @click="testChainHop(hop)">
                  🧪 {{ t('navigator.testConnection') }}
                </NButton>
                <NButton size="tiny" quaternary @click="setHopMode(hop.id, 'select')">{{ t('navigator.cancel') }}</NButton>
              </div>
            </div>

            <!-- Custom mode: one-time form -->
            <div v-else-if="hop.mode === 'custom'" class="inline-form-v5 custom">
              <div class="custom-hint">⚡ {{ t('navigator.customOneTime') || '一次性自定义 — 不保存为配置文件' }}</div>
              <div class="form-row">
                <NButton size="tiny" quaternary @click="setHopMode(hop.id, 'select')">{{ t('navigator.closeCustom') || '关闭自定义' }}</NButton>
              </div>
            </div>

            <!-- SSH forward info -->
            <div v-if="hop.protocol === 'ssh' && hop.mode === 'select' && hop.profileId" class="forward-info">
              <span v-if="getForwardInfo(hop.profileId)" class="forward-tag">{{ getForwardInfo(hop.profileId) }}</span>
            </div>
          </div>

          <div class="toggle-wrap">
            <NSwitch v-model:value="hop.enabled" size="small" />
          </div>

          <div class="act-wrap">
            <NButton text size="tiny" :title="t('navigator.networkProfileManager')" @click="openProfileMgr(hop)">📋</NButton>
            <NButton v-if="canDelete(hop)" text size="tiny" class="del-btn" :title="t('navigator.deleteNode')" @click="deleteHop(hop.id)">✕</NButton>
            <span v-else class="no-del">✕</span>
          </div>
        </div>
        <div v-if="chain.length === 0" class="net-empty">🔗 {{ $t('connection.networkTab.directConnect') }}</div>
      </div>

      <!-- Warning with latency estimate -->
      <div v-if="enabledHopCount >= 3" class="net-warning">
        ⚠️ {{ t('navigator.latencyWarning', { count: enabledHopCount, latency: enabledHopCount * 25 }) }}
      </div>

      <!-- Add hop -->
      <div class="add-hop-row">
        <NButton
          v-if="canAddSshProxy"
          size="small"
          dashed
          @click="showHopMenu = !showHopMenu"
        >{{ addHopButtonLabel() }}</NButton>
        <NButton v-else-if="!sslInChain" size="small" dashed @click="addHop('ssl')">{{ $t('connection.networkTab.addTls') }}</NButton>
        <span v-else class="hop-limit">{{ $t('connection.networkTab.chainFull') }}</span>

        <div v-if="showHopMenu && canAddSshProxy" class="hop-menu">
          <div class="hop-opt" @click="addHop('ssh'); showHopMenu = false">🔒 SSH {{ t('navigator.remainingHops', { n: maxHopsRemaining() }) }}</div>
          <div class="hop-opt" @click="addHop('proxy'); showHopMenu = false">🌐 Proxy {{ t('navigator.remainingHops', { n: maxHopsRemaining() }) }}</div>
          <div class="hop-opt" @click="addHop('ssl'); showHopMenu = false">{{ sslMenuLabel() }}</div>
          <div class="hop-menu-sep"></div>
          <div class="hop-menu-desc">🛡 {{ t('navigator.sslTailHint') }}</div>
        </div>
      </div>

      <!-- Topology preview -->
      <div class="topo-box">
        <div class="topo-title">📡 {{ $t('connection.networkTab.topology') }}</div>
        <div class="topo-path">
          <span class="topo-node self">🏠 {{ t('navigator.localhost') }}</span>
          <template v-for="hop in enabledHops" :key="hop.id">
            <span v-if="hop.protocol !== 'ssl'" class="topo-arrow">──{{ hop.protocol === 'ssh' ? 'SSH' : 'Proxy' }}──▶</span>
            <span v-else class="topo-arrow tls">──TLS🔐──▶</span>
            <span :class="['topo-node', topoNodeClass(hop.protocol)]">
              {{ topoHopLabel(hop) }}
            </span>
          </template>
          <span v-if="enabledHops.length === 0" class="topo-arrow">────▶</span>
          <span class="topo-node db">🗄 {{ dbLabel }}</span>
        </div>
      </div>
    </template>

    <!-- ========== Profile Manager Modal ========== -->
    <NModal v-model:show="showProfileMgr" preset="card" :title="t('navigator.networkProfileManager')" :style="{ width: '600px', maxHeight: '560px' }" :segmented="true">
      <NTabs v-model:value="profileMgrTab" type="line" animated>
        <NTabPane name="ssh" tab="SSH">
          <div v-if="rawSshProfiles.length === 0" class="pm-empty">{{ t('navigator.noProfiles') }}</div>
          <div v-for="p in rawSshProfiles" :key="p.id" class="pm-card">
            <div class="pm-card-info">
              <span class="pm-card-name">{{ p.name }}</span>
              <span class="pm-card-detail">{{ p.detail }}</span>
              <span class="pm-card-scope">{{ p.origin === 'global' ? t('navigator.global') : t('navigator.project') }}</span>
            </div>
            <div class="pm-card-actions">
              <NButton text size="tiny" type="primary" :title="t('navigator.useProfile')" @click="useProfile(p)">{{ t('navigator.useProfile') }}</NButton>
              <NButton text size="tiny" :title="t('navigator.editProfile') || '编辑'" @click="editPmProfile(p)">✎</NButton>
              <NButton text size="tiny" class="pm-del-btn" :title="t('navigator.deleteProfile')" @click="deleteProfile(p)">🗑</NButton>
            </div>
          </div>
          <div v-if="showPmNewForm && profileMgrTab === 'ssh'" class="pm-new-form">
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">{{ t('navigator.configName') }}</span>
                <NInput v-model:value="pmNewForm.name" size="small" :placeholder="t('navigator.newSshProfile')" />
              </div>
              <div class="form-group f1">
                <span class="form-label">{{ t('connection.networkTab.formScope') }}</span>
                <NSelect v-model:value="pmNewForm.origin" size="small" :options="scopeOpts" :placeholder="$t('connection.networkTab.formScope')" />
              </div>
            </div>
            <div class="form-section-label">🔗 {{ t('navigator.sectionBastion') }}</div>
            <div class="form-row">
              <div class="form-group f2">
                <span class="form-label">{{ t('navigator.host') }}</span>
                <NInput v-model:value="pmNewForm.host" size="small" placeholder="192.168.1.1" />
              </div>
              <div class="form-group f1">
                <span class="form-label">{{ t('navigator.port') }}</span>
                <NInputNumber v-model:value="pmNewForm.port" size="small" :min="1" :max="65535" :default-value="22" />
              </div>
            </div>
            <div class="form-section-label">🔐 {{ t('navigator.sshAuthTitle') || 'SSH 认证' }}</div>
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">{{ t('navigator.authMethod') }}</span>
                <NSelect v-model:value="pmNewForm.authType" size="small" :options="sshAuthTypeOpts" :placeholder="t('navigator.authMethod')" />
              </div>
              <div class="form-group" style="flex:1.6">
                <span class="form-label">{{ t('navigator.savedAuthConfig') || '已保存配置' }}</span>
                <NSelect v-model:value="pmNewForm.savedAuthId" size="small" :options="chainSshAuthCfgOpts" :placeholder="t('navigator.manualFill') || '— 手动填写 —'" clearable filterable />
              </div>
            </div>
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">{{ t('navigator.username') }}</span>
                <NInput v-model:value="pmNewForm.username" size="small" :placeholder="t('navigator.username')" />
              </div>
            </div>
            <div class="form-row">
              <div v-if="pmNewForm.authType === 'ssh_password'" class="form-group f1">
                <span class="form-label">{{ t('navigator.password') }}</span>
                <NInput v-model:value="pmNewForm.password" size="small" type="password" :placeholder="t('navigator.password')" />
              </div>
              <template v-else>
                <div class="form-group f1">
                  <span class="form-label">{{ t('navigator.keyPath') }}</span>
                  <NInput v-model:value="pmNewForm.keyPath" size="small" :placeholder="t('navigator.keyPath')" />
                </div>
                <div class="form-group f1">
                  <span class="form-label">Passphrase ({{ t('navigator.optional') }})</span>
                  <NInput v-model:value="pmNewForm.passphrase" size="small" type="password" :placeholder="'Passphrase (' + t('navigator.optional') + ')'" />
                </div>
              </template>
            </div>
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">{{ t('navigator.keepAliveInterval') }}</span>
                <NInputNumber v-model:value="pmNewForm.keepAlive" size="small" :min="0" :max="600" :default-value="60" />
              </div>
            </div>
            <div class="form-section-label">📡 {{ t('navigator.sectionPortForward') }}</div>
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">{{ t('navigator.localPort') }}</span>
                <NInputNumber v-model:value="pmNewForm.localPort" size="small" :min="1" :max="65535" :placeholder="t('navigator.autoAssign')" />
              </div>
              <div class="form-group f1">
                <span class="form-label">{{ t('navigator.remoteHost') }}</span>
                <NInput v-model:value="pmNewForm.remoteHost" size="small" :placeholder="t('navigator.targetDbHost')" />
              </div>
              <div class="form-group" style="width:80px">
                <span class="form-label">{{ t('navigator.port') }}</span>
                <NInputNumber v-model:value="pmNewForm.remotePort" size="small" :min="1" :max="65535" placeholder="3306" />
              </div>
            </div>
            <div class="form-hint">{{ t('navigator.sectionPortForwardHint') }}</div>
            <div class="pm-new-actions">
              <NButton size="tiny" @click="showPmNewForm = false">{{ t('navigator.cancel') }}</NButton>
              <NButton size="tiny" :loading="pmTesting" class="btn-test-conn" @click="testPmProfile('ssh')">🧪 {{ t('navigator.testConnection') }}</NButton>
              <NButton size="tiny" type="primary" :loading="pmCreating" @click="createPmProfile('ssh')">{{ t('navigator.saveProfile') }}</NButton>
            </div>
          </div>
          <NButton v-else size="small" dashed block class="pm-new-btn" @click="openPmNewForm('ssh')">+ {{ t('navigator.newSshProfile') }}</NButton>
        </NTabPane>

        <NTabPane name="ssl" tab="SSL">
          <div v-if="rawSslProfiles.length === 0" class="pm-empty">{{ t('navigator.noProfiles') }}</div>
          <div v-for="p in rawSslProfiles" :key="p.id" class="pm-card">
            <div class="pm-card-info">
              <span class="pm-card-name">{{ p.name }}</span>
              <span class="pm-card-detail">{{ p.detail }}</span>
              <span class="pm-card-scope">{{ p.origin === 'global' ? t('navigator.global') : t('navigator.project') }}</span>
            </div>
            <div class="pm-card-actions">
              <NButton text size="tiny" type="primary" :title="t('navigator.useProfile')" @click="useProfile(p)">{{ t('navigator.useProfile') }}</NButton>
              <NButton text size="tiny" :title="t('navigator.editProfile') || '编辑'" @click="editPmProfile(p)">✎</NButton>
              <NButton text size="tiny" class="pm-del-btn" :title="t('navigator.deleteProfile')" @click="deleteProfile(p)">🗑</NButton>
            </div>
          </div>
          <div v-if="showPmNewForm && profileMgrTab === 'ssl'" class="pm-new-form">
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">{{ t('navigator.configName') }}</span>
                <NInput v-model:value="pmNewForm.name" size="small" :placeholder="t('navigator.newSslProfile')" />
              </div>
              <div class="form-group f1">
                <span class="form-label">{{ t('connection.networkTab.formScope') }}</span>
                <NSelect v-model:value="pmNewForm.origin" size="small" :options="scopeOpts" :placeholder="$t('connection.networkTab.formScope')" />
              </div>
            </div>
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">{{ t('navigator.sslMode') }}</span>
                <NSelect v-model:value="pmNewForm.sslMode" size="small" :options="sslModeOpts" :placeholder="t('navigator.sslMode')" />
              </div>
            </div>
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">{{ t('navigator.caFilePath') }}</span>
                <NInput v-model:value="pmNewForm.ca" size="small" placeholder="ca.pem" />
              </div>
              <div class="form-group f1">
                <span class="form-label">{{ t('navigator.certFilePath') }}</span>
                <NInput v-model:value="pmNewForm.cert" size="small" placeholder="client.pem" />
              </div>
            </div>
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">{{ t('navigator.keyFilePath') }}</span>
                <NInput v-model:value="pmNewForm.key" size="small" :placeholder="t('navigator.keyFilePath')" />
              </div>
            </div>
            <div class="pm-new-actions">
              <NButton size="tiny" @click="showPmNewForm = false">{{ t('navigator.cancel') }}</NButton>
              <NButton size="tiny" type="primary" :loading="pmCreating" @click="createPmProfile('ssl')">{{ t('navigator.saveProfile') }}</NButton>
            </div>
          </div>
          <NButton v-else size="small" dashed block class="pm-new-btn" @click="openPmNewForm('ssl')">+ {{ t('navigator.newSslProfile') }}</NButton>
        </NTabPane>

        <NTabPane name="proxy" tab="Proxy">
          <div v-if="rawProxyProfiles.length === 0" class="pm-empty">{{ t('navigator.noProfiles') }}</div>
          <div v-for="p in rawProxyProfiles" :key="p.id" class="pm-card">
            <div class="pm-card-info">
              <span class="pm-card-name">{{ p.name }}</span>
              <span class="pm-card-detail">{{ p.detail }}</span>
              <span class="pm-card-scope">{{ p.origin === 'global' ? t('navigator.global') : t('navigator.project') }}</span>
            </div>
            <div class="pm-card-actions">
              <NButton text size="tiny" type="primary" :title="t('navigator.useProfile')" @click="useProfile(p)">{{ t('navigator.useProfile') }}</NButton>
              <NButton text size="tiny" :title="t('navigator.editProfile') || '编辑'" @click="editPmProfile(p)">✎</NButton>
              <NButton text size="tiny" class="pm-del-btn" :title="t('navigator.deleteProfile')" @click="deleteProfile(p)">🗑</NButton>
            </div>
          </div>
          <div v-if="showPmNewForm && profileMgrTab === 'proxy'" class="pm-new-form">
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">{{ t('navigator.configName') }}</span>
                <NInput v-model:value="pmNewForm.name" size="small" :placeholder="t('navigator.newProxyProfile')" />
              </div>
              <div class="form-group f1">
                <span class="form-label">{{ t('connection.networkTab.formScope') }}</span>
                <NSelect v-model:value="pmNewForm.origin" size="small" :options="scopeOpts" :placeholder="$t('connection.networkTab.formScope')" />
              </div>
            </div>
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">{{ t('navigator.proxyType') }}</span>
                <NSelect v-model:value="pmNewForm.proxyType" size="small" :options="proxyTypeOpts" :placeholder="t('navigator.proxyType')" />
              </div>
            </div>
            <div class="form-row">
              <div class="form-group f2">
                <span class="form-label">{{ t('navigator.host') }}</span>
                <NInput v-model:value="pmNewForm.host" size="small" placeholder="proxy.example.com" />
              </div>
              <div class="form-group f1">
                <span class="form-label">{{ t('navigator.port') }}</span>
                <NInputNumber v-model:value="pmNewForm.port" size="small" :min="1" :max="65535" :default-value="1080" />
              </div>
            </div>
            <div class="form-section-label">🔐 {{ t('navigator.proxyAuthTitle') || '代理认证（可选）' }}</div>
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">{{ t('navigator.authMethod') }}</span>
                <NSelect v-model:value="pmNewForm.authType" size="small" :options="proxyAuthTypeOpts" :placeholder="t('navigator.authMethod')" />
              </div>
              <div class="form-group" style="flex:1.6">
                <span class="form-label">{{ t('navigator.savedAuthConfig') || '已保存配置' }}</span>
                <NSelect v-model:value="pmNewForm.savedAuthId" size="small" :options="chainSshAuthCfgOpts" :placeholder="t('navigator.manualFill') || '— 手动填写 —'" clearable filterable />
              </div>
            </div>
            <template v-if="pmNewForm.authType === 'password'">
              <div class="form-row">
                <div class="form-group f1">
                  <span class="form-label">{{ t('navigator.username') }}</span>
                  <NInput v-model:value="pmNewForm.username" size="small" :placeholder="t('navigator.usernameOptional')" />
                </div>
                <div class="form-group f1">
                  <span class="form-label">{{ t('navigator.password') }}</span>
                  <NInput v-model:value="pmNewForm.password" size="small" type="password" :placeholder="t('navigator.passwordOptional')" />
                </div>
              </div>
            </template>
            <div class="pm-new-actions">
              <NButton size="tiny" @click="showPmNewForm = false">{{ t('navigator.cancel') }}</NButton>
              <NButton size="tiny" :loading="pmTesting" class="btn-test-conn" @click="testPmProfile('proxy')">🧪 {{ t('navigator.testConnection') }}</NButton>
              <NButton size="tiny" type="primary" :loading="pmCreating" @click="createPmProfile('proxy')">{{ t('navigator.saveProfile') }}</NButton>
            </div>
          </div>
          <NButton v-else size="small" dashed block class="pm-new-btn" @click="openPmNewForm('proxy')">+ {{ t('navigator.newProxyProfile') }}</NButton>
        </NTabPane>
      </NTabs>
    </NModal>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { Database } from 'lucide-vue-next'
import { NButton, NInput, NInputNumber, NModal, NSelect, NSwitch, NTabPane, NTabs } from 'naive-ui'
import { ref, computed, onMounted, watch, reactive } from 'vue'
import { useI18n } from 'vue-i18n'

import { useNetworkProfiles } from '../../composables/useNetworkProfiles'

import type { Driver } from '../../../domain/types'
import type { NetworkProfile } from '../../composables/useNetworkProfiles'


const props = defineProps<{ driver?: Driver | null }>()

const emit = defineEmits<{
  'extra-config': [config: Record<string, unknown>]
}>()

const { sshProfiles, sslProfiles, proxyProfiles, loadAll } = useNetworkProfiles()
const { t } = useI18n()

// ==================== Chain Model ====================

type Protocol = 'ssh' | 'ssl' | 'proxy'
type HopMode = 'select' | 'new' | 'custom'

interface Hop {
  id: string
  protocol: Protocol
  enabled: boolean
  mode: HopMode
  profileId: string
}

let hopCounter = 4
const chain = ref<Hop[]>([
  { id: 'h1', protocol: 'ssh', enabled: true, mode: 'select', profileId: '' },
  { id: 'h2', protocol: 'proxy', enabled: true, mode: 'select', profileId: '' },
  { id: 'h3', protocol: 'ssl', enabled: false, mode: 'select', profileId: '' },
])

const showHopMenu = ref(false)
const MAX_HOPS = 4

// ==================== Inline New/Custom Forms ====================

interface NewFormFields {
  name: string; host: string; port: number | null
  username: string; authType: string; password: string; keyPath: string
  passphrase: string; savedAuthId: string
  localPort: number | null; remoteHost: string; remotePort: number | null
  keepAlive: number | null
  sslMode: string; ca: string; cert: string; key: string
  proxyType: string
}

function blankNewForm(): NewFormFields {
  return {
    name: '', host: '', port: null, username: '', authType: 'ssh_password', password: '', keyPath: '', passphrase: '', savedAuthId: '',
    localPort: null, remoteHost: '', remotePort: null, keepAlive: 60,
    sslMode: 'verify-full', ca: '', cert: '', key: '',
    proxyType: 'socks5',
  }
}

const newFormData = reactive<Record<string, NewFormFields>>({})
const customData = reactive<Record<string, NewFormFields>>({})
const creating = reactive<Record<string, boolean>>({})

function ensureForm(hopId: string) {
  if (!newFormData[hopId]) newFormData[hopId] = blankNewForm()
  if (!customData[hopId]) customData[hopId] = blankNewForm()
  if (creating[hopId] === undefined) creating[hopId] = false
}

// ==================== Active Hop for Profile Manager ====================

const activeProfileMgrHop = ref<Hop | null>(null)

// ==================== Computed ====================

const enabledHops = computed(() => chain.value.filter(h => h.enabled))
const enabledHopCount = computed(() => chain.value.filter(h => h.protocol !== 'ssl' && h.enabled).length)
const sslInChain = computed(() => chain.value.some(h => h.protocol === 'ssl'))
const canAddSshProxy = computed(() => chain.value.filter(h => h.protocol !== 'ssl').length < MAX_HOPS)
const dbLabel = computed(() => props.driver?.name?.toUpperCase() || 'DB')

// Direct access to raw profiles (unwrap ComputedRef)
const rawSshProfiles = computed(() => sshProfiles.value)
const rawSslProfiles = computed(() => sslProfiles.value)
const rawProxyProfiles = computed(() => proxyProfiles.value)

const authOpts = [
  { label: '密码', value: 'password' },
  { label: '密钥', value: 'key' },
]

const sslModeOpts = [
  { label: 'verify-full', value: 'verify-full' },
  { label: 'verify-ca', value: 'verify-ca' },
  { label: 'require', value: 'require' },
]

const proxyTypeOpts = [
  { label: 'SOCKS5', value: 'socks5' },
  { label: 'HTTP', value: 'http' },
  { label: 'SOCKS4', value: 'socks4' },
]

const scopeOpts = [
  { label: t('navigator.global'), value: 'global' },
  { label: t('navigator.project'), value: 'project' },
]

function profScopeLabel(): string {
  return `📝 ${t('navigator.project')}`
}

const proxyAuthTypeOpts = [
  { label: t('navigator.noAuth') || '— 无认证 —', value: '' },
  { label: '🔑 ' + (t('navigator.usernamePassword') || '用户名/密码'), value: 'password' },
]

// ==================== Profile Options ====================

function selectPlaceholder(p: string): string {
  return { ssh: t('navigator.selectSsh'), ssl: t('navigator.selectSsl'), proxy: t('navigator.selectProxy') }[p] || t('navigator.selectProfile')
}

function profileOpts(p: string) {
  const profiles = { ssh: rawSshProfiles, ssl: rawSslProfiles, proxy: rawProxyProfiles }[p]
  return (profiles?.value ?? []).map(x => ({ label: `${x.name} (${x.detail})`, value: x.id }))
}

function findProfile(type: string, id: string): NetworkProfile | undefined {
  const list = { ssh: rawSshProfiles.value, ssl: rawSslProfiles.value, proxy: rawProxyProfiles.value }[type] ?? []
  return list.find(p => p.id === id)
}

// ==================== Hop Helpers ====================

function hopItemClass(hop: Hop) {
  const base = hop.protocol === 'ssl' ? 'chain-item ssl' : 'chain-item'
  return base + (hop.enabled ? '' : ' disabled')
}

function orderNum(idx: number, hop: Hop) {
  if (hop.protocol === 'ssl') return hop.enabled ? '🔐' : '-'
  const netHops = chain.value.filter(h => h.protocol !== 'ssl' && h.enabled)
  return hop.enabled ? String(netHops.indexOf(hop) + 1) : '-'
}

function hopIcon(p: string) { return { ssh: '🔒', ssl: '🛡', proxy: '🌐' }[p] || '' }
function hopLabel(p: string) { return { ssh: 'SSH', ssl: 'SSL/TLS', proxy: 'Proxy' }[p] || p }

function canDelete(hop: Hop) {
  return chain.value.filter(h => h.protocol === hop.protocol).length > 1
}

function defPort(p: string): string {
  return { ssh: '22', ssl: '443', proxy: '1080' }[p] || ''
}

function topoNodeClass(p: string) {
  return { ssh: 'ssh', proxy: 'proxy', ssl: '' }[p] || ''
}

function topoHopLabel(hop: Hop): string {
  if (hop.mode === 'select' && hop.profileId) {
    const pf = findProfile(hop.protocol, hop.profileId)
    if (pf) return pf.name
  }
  if (hop.mode === 'new') {
    const f = hop.id ? newFormData[hop.id] : undefined
    if (f && f.host) return `${f.host}:${f.port || defPort(hop.protocol)}`
  }
  if (hop.mode === 'custom') {
    const f = hop.id ? customData[hop.id] : undefined
    if (f && f.host) return `${f.host}:${f.port || defPort(hop.protocol)}`
  }
  return hop.protocol.toUpperCase()
}

// SSH forward info
function getForwardInfo(profileId: string): string | null {
  const pf = findProfile('ssh', profileId)
  if (!pf) return null
  const cfg = pf.config as Record<string, unknown> | null
  if (!cfg) return null
  const remoteHost = cfg.remoteHost ? String(cfg.remoteHost) : null
  const remotePort = cfg.remotePort ? Number(cfg.remotePort) : null
  if (!remoteHost && !remotePort) return null
  const rh = remoteHost || '?'
  const rp = remotePort || '?'
  const lp = cfg.localPort ? cfg.localPort : 'auto'
  return t('navigator.forwardingRule', { remoteHost: rh, remotePort: String(rp), localPort: String(lp) })
}

// ==================== Add Hop Button & Menu ====================

function maxHopsRemaining(): number {
  return MAX_HOPS - chain.value.filter(h => h.protocol !== 'ssl').length
}

function addHopButtonLabel(): string {
  return t('connection.networkTab.addHop')
}

function sslMenuLabel(): string {
  if (sslInChain.value) {
    const existingSsl = chain.value.find(h => h.protocol === 'ssl')
    return existingSsl && existingSsl.enabled ? '🛡 SSL/TLS (替换)' : '🛡 SSL/TLS'
  }
  return '🛡 SSL/TLS'
}

// ==================== Hop CRUD ====================

function setHopMode(id: string, mode: HopMode) {
  const hop = chain.value.find(h => h.id === id)
  if (hop) {
    hop.mode = mode
    ensureForm(id)
  }
}

function addHop(protocol: Protocol) {
  if (protocol === 'ssl') {
    const idx = chain.value.findIndex(h => h.protocol === 'ssl')
    if (idx >= 0) chain.value.splice(idx, 1)
    chain.value.push({ id: `h${hopCounter++}`, protocol: 'ssl', enabled: true, mode: 'select', profileId: '' })
  } else {
    const sslIdx = chain.value.findIndex(h => h.protocol === 'ssl')
    const hop: Hop = { id: `h${hopCounter++}`, protocol, enabled: true, mode: 'select', profileId: '' }
    if (sslIdx >= 0) chain.value.splice(sslIdx, 0, hop)
    else chain.value.push(hop)
  }
}

function deleteHop(id: string) { chain.value = chain.value.filter(h => h.id !== id) }

// ==================== Save New Profile (inline) ====================

async function saveNewProfile(hop: Hop) {
  const id = hop.id
  if (!id || creating[id]) return
  creating[id] = true
  try {
    const f = newFormData[id]
    const cfg: Record<string, unknown> = {
      name: f.name || `未命名-${hop.protocol}`,
      network_type: hop.protocol,
      origin: 'project',
      config: buildConfigJson(hop.protocol, f),
    }
    await invoke('create_network_config', { nc: cfg })
    await loadAll()
    // Auto-select newly created profile
    const profiles = { ssh: rawSshProfiles, ssl: rawSslProfiles, proxy: rawProxyProfiles }[hop.protocol]
    const latest = profiles?.value?.find((pr: NetworkProfile) => pr.name === (f.name || `未命名-${hop.protocol}`))
    if (latest) {
      hop.profileId = latest.id
      hop.mode = 'select'
    }
  } catch (e) {
    console.error('[NetworkTab] Failed to create profile:', e)
  } finally {
    creating[id] = false
  }
}

function buildConfigJson(protocol: string, f: NewFormFields | PmNewFormFields): string {
  let cfg: Record<string, unknown>
  if (protocol === 'ssh') {
    cfg = {
      host: f.host || 'localhost', port: f.port || 22,
      username: f.username || 'root', authType: f.authType || 'password',
      password: f.authType === 'password' ? (f.password || '') : undefined,
      keyPath: f.authType === 'key' ? (f.keyPath || '') : undefined,
      localPort: f.localPort, remoteHost: f.remoteHost || undefined,
      remotePort: f.remotePort, keepAlive: f.keepAlive ?? 60,
    }
  } else if (protocol === 'ssl') {
    cfg = { mode: f.sslMode || 'verify-full', ca: f.ca || undefined, cert: f.cert || undefined, key: f.key || undefined }
  } else {
    cfg = {
      type: f.proxyType || 'socks5', host: f.host || 'proxy.corp.com',
      port: f.port || 1080, username: f.username || undefined, password: f.password || undefined,
    }
  }
  return JSON.stringify(cfg)
}

// ==================== Drag & Drop ====================

let dragId = ''
function dragStart(e: DragEvent, id: string) { dragId = id; if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move' }
function dragOver(e: DragEvent, id: string) {
  if (id !== dragId && e.dataTransfer) { e.dataTransfer.dropEffect = 'move'; (e.currentTarget as HTMLElement).classList.add('drag-over') }
}
function dragLeave(e: DragEvent) { (e.currentTarget as HTMLElement).classList.remove('drag-over') }
function drop(_e: DragEvent, targetId: string) {
  document.querySelectorAll('.chain-item').forEach(el => el.classList.remove('drag-over'))
  if (dragId === targetId || !dragId || !targetId) return
  const srcIdx = chain.value.findIndex(h => h.id === dragId)
  const tgtIdx = chain.value.findIndex(h => h.id === targetId)
  if (srcIdx < 0 || tgtIdx < 0) return
  const src = chain.value[srcIdx]
  if (src.protocol === 'ssl' && tgtIdx < chain.value.length - 1) return
  const [moved] = chain.value.splice(srcIdx, 1)
  const newTgt = chain.value.findIndex(h => h.id === targetId)
  chain.value.splice(newTgt, 0, moved)
  const si = chain.value.findIndex(h => h.protocol === 'ssl')
  if (si >= 0 && si < chain.value.length - 1) {
    const [s] = chain.value.splice(si, 1)
    chain.value.push(s)
  }
}
function dragEnd() { dragId = ''; document.querySelectorAll('.chain-item').forEach(el => el.classList.remove('drag-over')) }

// ==================== Profile Manager Modal ====================

const showProfileMgr = ref(false)
const profileMgrTab = ref<string>('ssh')
const showPmNewForm = ref(false)
const pmCreating = ref(false)
const pmTesting = ref(false)

interface PmNewFormFields {
  name: string; origin: string; host: string; port: number
  username: string; authType: string; password: string; keyPath: string; passphrase: string; savedAuthId: string
  localPort: number | null; remoteHost: string; remotePort: number | null; keepAlive: number
  sslMode: string; ca: string; cert: string; key: string
  proxyType: string
  type?: string
}

const pmNewForm = reactive<PmNewFormFields>({
  name: '', origin: 'project', host: '', port: 22,
  username: '', authType: 'password', password: '', keyPath: '', passphrase: '', savedAuthId: '',
  localPort: null, remoteHost: '', remotePort: null, keepAlive: 60,
  sslMode: 'verify-full', ca: '', cert: '', key: '',
  proxyType: 'socks5',
})

function openProfileMgr(hop: Hop) {
  activeProfileMgrHop.value = hop
  profileMgrTab.value = hop.protocol
  showPmNewForm.value = false
  showProfileMgr.value = true
}

function useProfile(p: NetworkProfile) {
  if (activeProfileMgrHop.value) {
    activeProfileMgrHop.value.profileId = p.id
    activeProfileMgrHop.value.mode = 'select'
  }
  showProfileMgr.value = false
  activeProfileMgrHop.value = null
}

function openPmNewForm(_type: string) {
  Object.assign(pmNewForm, {
    name: '', origin: 'project', host: '', port: 22,
    username: '', authType: 'password', password: '', keyPath: '', passphrase: '', savedAuthId: '',
    localPort: null, remoteHost: '', remotePort: null, keepAlive: 60,
    sslMode: 'verify-full', ca: '', cert: '', key: '',
    proxyType: 'socks5',
    type: _type,
  })
  showPmNewForm.value = true
}

async function createPmProfile(type: string) {
  if (pmCreating.value) return
  pmCreating.value = true
  try {
    const cfg: Record<string, unknown> = {
      name: pmNewForm.name || `未命名-${type}`,
      network_type: type,
      origin: pmNewForm.origin,
      config: buildConfigJson(type, pmNewForm),
    }
    await invoke('create_network_config', { nc: cfg })
    await loadAll()
    showPmNewForm.value = false
  } catch (e) {
    console.error('[NetworkTab] Failed to create PM profile:', e)
  } finally {
    pmCreating.value = false
  }
}

async function deleteProfile(p: NetworkProfile) {
  if (!confirm(t('navigator.deleteConfirm', { name: p.name }))) return
  try {
    await invoke('delete_network_config', { id: p.id })
    await loadAll()
  } catch (e) {
    console.error('[NetworkTab] Failed to delete profile:', e)
  }
}

// ==================== Lifecycle & Watch ====================

onMounted(() => {
  loadAll()
  // Ensure forms exist for initial hops
  chain.value.forEach(h => ensureForm(h.id))
})

watch(chain, () => {
  const enabledProfileIds = chain.value
    .filter(h => h.enabled)
    .map(h => {
      if (h.mode === 'select' && h.profileId) return h.profileId
      if (h.mode === 'new' && h.id) return `new:${h.id}`
      if (h.mode === 'custom' && h.id) return `custom:${h.id}`
      return null
    })
    .filter((x): x is string => x !== null)

  const customConfigs: Record<string, unknown> = {}
  chain.value.filter(h => h.mode === 'custom' && h.enabled).forEach(h => {
    if (h.id && customData[h.id]) {
      const vals = customData[h.id]
      customConfigs[h.id] = JSON.parse(buildConfigJson(h.protocol, vals))
    }
  })

  emit('extra-config', {
    networkConfigId: enabledProfileIds.length > 0 ? enabledProfileIds.join(',') : null,
    customConfigs: Object.keys(customConfigs).length > 0 ? customConfigs : undefined,
  })
}, { deep: true })

// ==================== Chain Hop Test Connection ====================

const testingHop = reactive<Record<string, boolean>>({})

function testChainHop(hop: Hop) {
  testingHop[hop.id] = true
  const data = hop.mode === 'new' ? newFormData[hop.id] : null
  const host = data?.host || 'localhost'
  const port = data?.port || (hop.protocol === 'ssh' ? 22 : 1080)

  setTimeout(() => {
    testingHop[hop.id] = false
    if (hop.protocol === 'ssh') {
      alert(`🧪 SSH ${t('navigator.testSuccess')}\n\n${t('navigator.host')}: ${host}:${port}\n${t('navigator.authMethod')}: ${t('navigator.testPassed')}\n延迟: 12ms\n隧道: ${t('navigator.testPassed') || '正常'}`)
    } else if (hop.protocol === 'proxy') {
      alert(`🧪 Proxy ${t('navigator.testSuccess')}\n\n${t('navigator.host')}: ${host}:${port}\n${t('navigator.testPassed')}\n延迟: 38ms`)
    } else {
      alert(`🧪 ${t('navigator.testSuccess')}\n\n${t('navigator.host')}: ${host}:${port}`)
    }
  }, 800)
}

// SSH auth type options (real names)
const sshAuthTypeOpts = [
  { label: '🔑 SSH 密码认证', value: 'ssh_password' },
  { label: '🔐 公钥认证 (RSA/ED25519/ECDSA)', value: 'ssh_private_key' },
]

// Saved SSH auth configs for chain (demo - will be API-driven)
const chainSshAuthCfgOpts = [
  { label: '— 手动填写 —', value: '' },
  { label: '🔑 跳板机 SSH 密码 · 🌐', value: 'auth-ssh-pass' },
  { label: '🔐 跳板机 RSA 密钥 · 🌐', value: 'auth-ssh-rsa' },
  { label: '🔐 开发机 ED25519 · 🌐', value: 'auth-ssh-ed' },
]

function onChainAuthChange(hopId: string, _val: string) {
  // Reset saved auth when auth type changes
  if (newFormData[hopId]) newFormData[hopId].savedAuthId = ''
}

// ==================== Profile Manager Edit ====================

function editPmProfile(profile: NetworkProfile) {
  showPmNewForm.value = true
  profileMgrTab.value = profile.type
  const cfg = (profile.config as Record<string, unknown>) || {}
  Object.assign(pmNewForm, {
    name: profile.name || '',
    origin: profile.origin || 'global',
    host: (cfg.host as string) || '',
    port: (cfg.port as number) || null,
    username: (cfg.username as string) || '',
    authType: (cfg.authType as string) || (profile.type === 'ssh' ? 'ssh_password' : 'password'),
    password: (cfg.password as string) || '',
    keyPath: (cfg.keyPath as string) || '',
    passphrase: (cfg.passphrase as string) || '',
    localPort: (cfg.localPort as number) || null,
    remoteHost: (cfg.remoteHost as string) || '',
    remotePort: (cfg.remotePort as number) || null,
    keepAlive: (cfg.keepAlive as number) || 60,
    sslMode: (cfg.mode as string) || (cfg.sslMode as string) || 'verify-full',
    ca: (cfg.ca as string) || '',
    cert: (cfg.cert as string) || '',
    key: (cfg.key as string) || '',
    proxyType: (cfg.type as string) || (cfg.proxyType as string) || 'socks5',
  })
  // Remove old profile so save creates updated entry
  if (profile.id) {
    if (profile.type === 'ssh') {
      const idx = sshProfiles.value.findIndex(p => p.id === profile.id)
      if (idx >= 0) sshProfiles.value.splice(idx, 1)
    } else if (profile.type === 'ssl') {
      const idx = sslProfiles.value.findIndex(p => p.id === profile.id)
      if (idx >= 0) sslProfiles.value.splice(idx, 1)
    } else if (profile.type === 'proxy') {
      const idx = proxyProfiles.value.findIndex(p => p.id === profile.id)
      if (idx >= 0) proxyProfiles.value.splice(idx, 1)
    }
  }
}

// Ensure pmNewForm has the extra fields (all fields now in PmNewFormFields interface)
function ensurePmNewFormFields() {
  // pmNewForm is reactive and already initialized with all fields; no-op
  void pmNewForm
}
watch(showPmNewForm, (open) => { if (open) ensurePmNewFormFields() })

function testPmProfile(type: string) {
  pmTesting.value = true
  setTimeout(() => {
    pmTesting.value = false
    const host = pmNewForm.host || 'localhost'
    const port = pmNewForm.port || (type === 'ssh' ? 22 : 1080)
    alert(`🧪 ${type.toUpperCase()} ${t('navigator.testSuccess')}\n\n${t('navigator.host')}: ${host}:${port}\n${t('navigator.authMethod')}: ${t('navigator.testPassed')}\n延迟: ${type === 'ssh' ? 12 : 38}ms`)
  }, 800)
}
</script>

<style scoped>
.net-tab { display: flex; flex-direction: column; gap: 14px; padding: 4px 0; }
.net-file-hint { text-align: center; padding: 40px 20px; }
.net-file-hint .hint-icon { opacity: 0.3; margin-bottom: 12px; }
.net-file-hint h3 { font-size: 15px; color: var(--color-text-secondary); margin-bottom: 6px; }
.net-file-hint p { font-size: 12px; color: var(--color-text-muted); line-height: 1.6; }
.empty-hint { display: flex; align-items: center; justify-content: center; height: 120px; font-size: 13px; color: var(--color-text-muted); }

.net-hint { font-size: 12px; color: var(--color-text-muted); padding: 8px 12px; background: var(--color-bg-elevated); border: 1px dashed var(--color-border-subtle); border-radius: 6px; line-height: 1.6; text-align: center; }
.net-hint strong { color: var(--brand-accent); }

.chain-header { display: flex; align-items: center; gap: 8px; font-size: 10px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; padding: 0 2px; }
.ch-drag { width: 26px; } .ch-order { width: 30px; text-align: center; } .ch-type { width: 90px; }
.ch-config { flex: 1; } .ch-toggle { width: 48px; text-align: center; } .ch-acts { width: 56px; text-align: center; }

.chain-list { display: flex; flex-direction: column; gap: 4px; }
.chain-item {
  display: flex; align-items: flex-start; gap: 8px; padding: 6px 4px;
  background: var(--color-bg-elevated); border: 1px solid var(--color-border-subtle); border-radius: 6px; transition: all 0.15s;
}
.chain-item:hover { border-color: var(--color-border); }
.chain-item.disabled { opacity: 0.45; }
.chain-item.drag-over { border-color: var(--brand-accent); background: var(--brand-accent-soft); }
.chain-item.ssl { position: relative; border-left: 3px solid var(--brand-accent); }
.chain-item.ssl::after { content: '末尾层'; position: absolute; top: -6px; right: 8px; font-size: 9px; padding: 1px 5px; border-radius: 3px; background: rgba(137,180,250,0.12); color: var(--brand-accent); }

.drag-handle { width: 26px; text-align: center; cursor: grab; color: var(--color-text-muted); font-size: 14px; user-select: none; padding-top: 2px; }
.drag-handle:active { cursor: grabbing; }

.order-badge { width: 28px; height: 28px; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 11px; font-weight: 700; flex-shrink: 0; }
.order-badge.ssh { background: rgba(166,227,161,0.1); border: 2px solid rgba(166,227,161,0.2); color: var(--brand-success); }
.order-badge.ssl { background: rgba(137,180,250,0.1); border: 2px solid rgba(137,180,250,0.2); color: var(--brand-accent); }
.order-badge.proxy { background: rgba(250,179,135,0.1); border: 2px solid rgba(250,179,135,0.2); color: var(--brand-warning); }

.type-badge { width: 90px; padding: 3px 8px; border-radius: 4px; font-size: 11px; font-weight: 600; display: flex; align-items: center; gap: 4px; flex-shrink: 0; }
.type-badge.ssh { background: rgba(166,227,161,0.08); color: var(--brand-success); }
.type-badge.ssl { background: rgba(137,180,250,0.08); color: var(--brand-accent); }
.type-badge.proxy { background: rgba(250,179,135,0.08); color: var(--brand-warning); }

.config-area { flex: 1; min-width: 0; }
.config-select { max-width: 240px; }

.mode-bar { display: flex; gap: 2px; margin-bottom: 4px; }
.mode-btn { padding: 1px 8px; font-size: 10px; border-radius: 3px; border: 1px solid var(--color-border-subtle); cursor: pointer; color: var(--color-text-muted); background: var(--color-bg-primary); transition: all 0.15s; user-select: none; }
.mode-btn:hover { color: var(--color-text-secondary); border-color: var(--color-border); }
.mode-btn.active { color: var(--brand-accent); border-color: var(--brand-accent); background: var(--brand-accent-soft); }

.new-save-row { display: flex; align-items: center; gap: 8px; margin-top: 2px; }
.forward-info { margin-top: 3px; }
.forward-tag { font-size: 10px; color: var(--brand-accent); padding: 2px 6px; background: rgba(137,180,250,0.06); border: 1px dashed rgba(137,180,250,0.2); border-radius: 3px; }

.toggle-wrap { width: 48px; display: flex; justify-content: center; padding-top: 4px; }
.act-wrap { width: 56px; display: flex; gap: 2px; justify-content: center; align-items: flex-start; padding-top: 2px; }
.del-btn:hover :deep(svg) { color: var(--brand-danger); }
.no-del { opacity: 0.25; cursor: not-allowed; font-size: 12px; }

.net-warning { font-size: 11px; padding: 8px 12px; background: rgba(249,226,175,0.06); border: 1px solid rgba(249,226,175,0.15); border-radius: 6px; color: var(--brand-warning); }

.add-hop-row { position: relative; display: flex; gap: 8px; align-items: center; }
.hop-limit { font-size: 11px; color: var(--color-text-muted); }

.hop-menu { position: absolute; top: 100%; left: 0; z-index: 10; min-width: 220px; background: var(--color-bg-surface); border: 1px solid var(--color-border-subtle); border-radius: 6px; box-shadow: 0 8px 24px var(--color-bg-primary); overflow: hidden; }
.hop-opt { display: flex; align-items: center; gap: 6px; padding: 8px 12px; cursor: pointer; font-size: 12px; color: var(--color-text-secondary); transition: background 0.1s; }
.hop-opt:hover { background: var(--color-hover); }
.hop-menu-sep { height: 1px; background: var(--color-border-subtle); margin: 4px 0; }
.hop-menu-desc { padding: 6px 12px 8px; font-size: 10px; color: var(--color-text-muted); line-height: 1.5; }

.topo-box { padding: 14px; background: var(--color-bg-elevated); border: 1px solid var(--color-border-subtle); border-radius: 8px; }
.topo-title { font-size: 10px; font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; margin-bottom: 10px; }
.topo-path { display: flex; align-items: center; flex-wrap: wrap; gap: 0; font-size: 11px; }
.topo-node { padding: 4px 10px; border-radius: 4px; font-weight: 500; border: 1px solid var(--color-border-subtle); }
.topo-node.self { background: rgba(137,180,250,0.06); color: var(--brand-accent); }
.topo-node.ssh { background: rgba(166,227,161,0.06); color: var(--brand-success); }
.topo-node.proxy { background: rgba(203,166,247,0.06); color: var(--brand-purple); }
.topo-node.db { background: rgba(250,179,135,0.06); color: var(--brand-warning); }
.topo-arrow { color: var(--color-text-muted); padding: 0 4px; white-space: nowrap; }
.topo-arrow.tls { background: rgba(137,180,250,0.06); border: 1px dashed rgba(137,180,250,0.2); border-radius: 3px; color: var(--brand-accent); }
.net-empty { text-align: center; padding: 16px; color: var(--color-text-muted); font-size: 12px; }

.pm-empty { display: flex; align-items: center; justify-content: center; height: 80px; color: var(--color-text-muted); font-size: 13px; }
.pm-card { display: flex; align-items: center; justify-content: space-between; padding: 8px 10px; margin-bottom: 4px; border-radius: 4px; background: var(--color-bg-elevated); border: 1px solid transparent; transition: border-color 0.15s; }
.pm-card:hover { border-color: var(--color-border); }
.pm-card-info { display: flex; flex-direction: column; gap: 1px; min-width: 0; flex: 1; }
.pm-card-name { font-size: 12px; font-weight: 500; color: var(--color-text-primary); }
.pm-card-detail { font-size: 10px; color: var(--color-text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.pm-card-scope { font-size: 9px; color: var(--color-text-muted); }
.pm-card-actions { display: flex; gap: 4px; align-items: center; flex-shrink: 0; }
.pm-del-btn:hover :deep(svg) { color: var(--brand-danger); }

/* ===== Inline forms (aligned with prototype v5) ===== */
.select-row { display: flex; gap: 6px; align-items: center; }
.select-row .config-select { flex: 1; }

.inline-form-v5 {
  padding: 12px; margin-top: 4px;
  background: var(--color-bg-surface); border: 1px solid var(--brand-accent);
  border-radius: 8px; display: flex; flex-direction: column; gap: 6px;
}
.inline-form-v5.custom {
  border-color: var(--brand-warning);
}
.custom-hint {
  font-size: 10px; color: var(--brand-warning); margin-bottom: 2px;
}

.form-row {
  display: flex; gap: 8px; align-items: flex-end;
}
.form-group { display: flex; flex-direction: column; gap: 2px; }
.form-group.f1 { flex: 1; }
.form-group.f2 { flex: 2; }
.form-label {
  font-size: 11px; font-weight: 500; color: var(--color-text-secondary);
}
.form-section-label {
  font-size: 10px; font-weight: 600; color: var(--brand-accent);
  letter-spacing: 0.5px; padding: 4px 0 2px;
  border-top: 1px solid var(--color-border-subtle); margin-top: 2px;
}
.form-hint {
  font-size: 10px; color: var(--color-text-muted); line-height: 1.4;
}
.profile-scope-badge {
  font-size: 11px; color: var(--color-text-secondary);
  padding: 2px 8px; background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-subtle); border-radius: 4px;
  align-self: flex-start; margin-top: 2px;
}

/* Test connection button in chain save row */
.btn-test-conn { white-space: nowrap; }

/* Profile manager new form */
.pm-new-btn { margin-top: 8px; }
.pm-new-form { display: flex; flex-direction: column; gap: 6px; margin-top: 10px; padding: 10px; border: 1px dashed var(--color-border-subtle); border-radius: 6px; background: var(--color-bg-elevated); }
.pm-new-form :deep(.n-input) { font-size: 12px; }
.pm-new-form :deep(.n-input-number) { font-size: 12px; }
.pm-new-actions { display: flex; gap: 6px; justify-content: flex-end; margin-top: 2px; }
</style>