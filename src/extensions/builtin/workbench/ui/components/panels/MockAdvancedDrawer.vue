<template>
  <NDrawer
    :show="visible"
    :width="420"
    placement="right"
    :on-update:show="onClose"
  >
    <NDrawerContent :title="`${columnName} 列 · 高级配置`" closable>
      <div class="drawer-body">
        <div class="field-header">
          <div class="field-row">
            <span class="field-label">字段名</span>
            <NInput v-model:value="localFieldName" size="small" style="flex:1" />
          </div>
          <div class="field-row">
            <span class="field-label">类型</span>
            <NSelect v-model:value="localDataType" size="small" :options="dataTypeOptions" style="flex:1" />
          </div>
        </div>

        <div class="section-title">数据类型筛选</div>
        <div class="filter-tabs">
          <NButton
            v-for="tab in filterTabs"
            :key="tab.key"
            :type="activeFilter === tab.key ? 'primary' : 'default'"
            size="tiny"
            :secondary="activeFilter !== tab.key"
            @click="activeFilter = tab.key"
          >
            {{ tab.label }}
          </NButton>
        </div>

        <div class="section-title">生成器选择</div>
        <div class="generator-list">
          <div
            v-for="gen in filteredGenerators"
            :key="gen.type"
            :class="['generator-item', { selected: localGeneratorType === gen.type }]"
            @click="selectGenerator(gen)"
          >
            <div class="gen-info">
              <span v-if="gen.recommended" class="recommend-badge">🟢 推荐</span>
              <span class="gen-name">{{ gen.label }}</span>
              <span class="gen-type">{{ gen.type }}</span>
            </div>
            <span class="gen-example">{{ gen.example }}</span>
          </div>
          <div v-if="filteredGenerators.length === 0" class="empty-list">
            该分类暂无可用生成器
          </div>
        </div>

        <NDivider style="margin: 12px 0" />

        <div v-if="paramFields.length > 0" class="params-section">
          <div class="section-title">参数配置</div>
          <div class="params-form">
            <div v-for="field in paramFields" :key="field.name" class="param-field">
              <label class="param-field-label">{{ field.label }}</label>

              <NInputNumber
                v-if="field.type === 'number'"
                :value="(localParams[field.name] as number) ?? (field.default as number)"
                size="small"
                :min="field.min"
                :max="field.max"
                @update:value="(v: number | null) => setParam(field.name, v ?? field.default)"
              />

              <NInput
                v-if="field.type === 'text'"
                :value="(localParams[field.name] as string) ?? ''"
                size="small"
                @update:value="(v: string) => setParam(field.name, v)"
              />

              <NSwitch
                v-if="field.type === 'boolean'"
                :value="(localParams[field.name] as boolean) ?? false"
                @update:value="(v: boolean) => setParam(field.name, v)"
              />
            </div>
          </div>
        </div>

        <div v-else class="empty-params">
          此生成器无需额外配置参数
        </div>

        <div v-if="showTimeSeries" class="timeseries-section">
          <NDivider style="margin: 12px 0" />
          <div class="section-title">时序关联（可选）</div>
          <div class="ts-row">
            <span class="ts-label">关联日期列</span>
            <NSelect v-model:value="tsDateColumn" size="small" :options="dateColumnOptions" placeholder="选择日期列" style="flex:1" />
          </div>
          <div v-if="tsDateColumn" class="ts-row">
            <span class="ts-label">趋势方向</span>
            <NSelect v-model:value="tsTrend" size="small" :options="trendOptions" style="flex:1" />
          </div>
          <div v-if="tsDateColumn && tsTrend === 'cycle'" class="ts-row">
            <span class="ts-label">周期长度(天)</span>
            <NInputNumber v-model:value="tsCycleLength" size="small" :min="1" style="flex:1" />
          </div>
          <div v-if="tsDateColumn" class="ts-row">
            <span class="ts-label">波动幅度(%)</span>
            <NInputNumber v-model:value="tsVolatility" size="small" :min="0" :max="100" style="flex:1" />
          </div>
        </div>

        <NDivider style="margin: 12px 0" />
        <div class="section-title">其他选项</div>
        <div class="other-options">
          <div class="opt-row">
            <span class="opt-label">允许空值比例</span>
            <NInputNumber v-model:value="localNullRatio" size="small" :min="0" :max="1" :step="0.05" style="width:80px" />
            <span class="opt-pct">{{ Math.round(localNullRatio * 100) }}%</span>
          </div>
          <div class="opt-row">
            <span class="opt-label">唯一值</span>
            <NSwitch v-model:value="localUnique" />
          </div>
        </div>

        <div class="source-info">
          生成器来源: fake-rs · {{ generatorModule }}
        </div>
      </div>

      <template #footer>
        <div class="drawer-footer">
          <NButton size="small" quaternary @click="restoreDefault">恢复智能默认</NButton>
          <div class="footer-right">
            <NButton size="small" @click="onClose">取消</NButton>
            <NButton type="primary" size="small" @click="onApply">应用</NButton>
          </div>
        </div>
      </template>
    </NDrawerContent>
  </NDrawer>
</template>

<script setup lang="ts">
import {
  NDrawer, NDrawerContent, NDivider, NButton, NInput, NInputNumber, NSelect, NSwitch,
} from 'naive-ui'
import { ref, computed, watch } from 'vue'

import type { GeneratorType, ColumnDataType } from '@/shared/api/mock-api'

interface ParamFieldDef {
  name: string
  label: string
  type: 'number' | 'text' | 'boolean'
  default: unknown
  min?: number
  max?: number
}

interface GeneratorDesc {
  type: GeneratorType
  label: string
  category: 'numeric' | 'date' | 'text' | 'boolean' | 'foreign_key'
  subCategory?: string
  recommended: boolean
  example: string
}

const GENERATOR_LIST: GeneratorDesc[] = [
  { type: 'auto_increment', label: '自增序列', category: 'numeric', recommended: false, example: '1, 2, 3, 4...' },
  { type: 'random_int', label: '均匀分布 (Range)', category: 'numeric', recommended: false, example: '4321, 8765, 234...' },
  { type: 'normal', label: '正态分布 (Normal)', category: 'numeric', recommended: false, example: '487, 512, 498...' },
  { type: 'log_normal', label: '对数正态 (LogNormal)', category: 'numeric', recommended: false, example: '234, 1567, 890...' },
  { type: 'random_walk', label: '随机游走 (Random Walk)', category: 'numeric', recommended: false, example: '1000→1012→1005...' },
  { type: 'random_float', label: '随机浮点数', category: 'numeric', recommended: false, example: '0.42, 0.87, 0.15...' },
  { type: 'random_decimal', label: '随机小数', category: 'numeric', recommended: false, example: '123.45, 678.90...' },
  { type: 'digit', label: '随机数字字符串', category: 'numeric', recommended: false, example: '38472, 10934...' },
  { type: 'number_with_format', label: '带格式数字', category: 'numeric', recommended: false, example: '123-45-6789' },

  { type: 'datetime', label: '随机日期时间', category: 'date', recommended: true, example: '2025-06-15 14:30:00' },
  { type: 'datetime_between', label: '指定范围', category: 'date', recommended: false, example: '2025-03-15, 2026-09-22...' },
  { type: 'datetime_before', label: '早于指定日期', category: 'date', recommended: false, example: '2025-12-01, 2024-03-18...' },
  { type: 'datetime_after', label: '晚于指定日期', category: 'date', recommended: false, example: '2025-06-15, 2026-02-28...' },
  { type: 'sequential_date', label: '递增序列', category: 'date', recommended: false, example: '01-01→01-02→01-03...' },
  { type: 'sequential_date_with_gaps', label: '含缺失间隔', category: 'date', recommended: false, example: '01-01 → gap → 01-03...' },
  { type: 'date', label: '纯日期', category: 'date', recommended: false, example: '2025-06-15' },
  { type: 'time', label: '纯时间', category: 'date', recommended: false, example: '14:30:00' },
  { type: 'duration', label: '时长', category: 'date', recommended: false, example: '+3600s, -7200s' },

  { type: 'name', label: '姓名', category: 'text', subCategory: 'personal', recommended: false, example: '张三, 李四, 王五' },
  { type: 'name_with_title', label: '姓名(带称谓)', category: 'text', subCategory: 'personal', recommended: false, example: 'Mr. John Smith' },
  { type: 'first_name', label: '名', category: 'text', subCategory: 'personal', recommended: false, example: '张, 李, 王' },
  { type: 'last_name', label: '姓', category: 'text', subCategory: 'personal', recommended: false, example: '三, 四, 五' },
  { type: 'title', label: '头衔', category: 'text', subCategory: 'personal', recommended: false, example: 'Dr., Mr., Mrs.' },
  { type: 'suffix', label: '姓名后缀', category: 'text', subCategory: 'personal', recommended: false, example: 'Jr., Sr., III' },
  { type: 'phone_number', label: '手机号', category: 'text', subCategory: 'personal', recommended: false, example: '13812345678' },
  { type: 'cell_number', label: '手机号(国际)', category: 'text', subCategory: 'personal', recommended: false, example: '1-555-123-4567' },
  { type: 'email', label: '邮箱', category: 'text', subCategory: 'personal', recommended: false, example: 'user@example.com' },
  { type: 'safe_email', label: '安全邮箱', category: 'text', subCategory: 'personal', recommended: false, example: 'zhangsan@example.com' },
  { type: 'free_email', label: '免费邮箱', category: 'text', subCategory: 'personal', recommended: false, example: 'lisi@gmail.com' },
  { type: 'free_email_provider', label: '邮箱服务商', category: 'text', subCategory: 'personal', recommended: false, example: 'gmail.com, outlook.com' },
  { type: 'domain_suffix', label: '域名后缀', category: 'text', subCategory: 'personal', recommended: false, example: 'com, org, net' },
  { type: 'username', label: '用户名', category: 'text', subCategory: 'personal', recommended: false, example: 'zhangsan_1985' },
  { type: 'password', label: '密码', category: 'text', subCategory: 'personal', recommended: false, example: 'aB3$xK7!mP' },

  { type: 'company_name', label: '公司名称', category: 'text', subCategory: 'company', recommended: false, example: '腾讯科技有限公司' },
  { type: 'company_suffix', label: '公司后缀', category: 'text', subCategory: 'company', recommended: false, example: 'Inc, LLC, Ltd' },
  { type: 'industry', label: '行业', category: 'text', subCategory: 'company', recommended: false, example: '信息技术, 金融服务' },
  { type: 'profession', label: '职位', category: 'text', subCategory: 'company', recommended: false, example: '软件工程师, 市场经理' },
  { type: 'seniority', label: '职级', category: 'text', subCategory: 'company', recommended: false, example: '初级, 高级, 总监' },
  { type: 'field', label: '领域', category: 'text', subCategory: 'company', recommended: false, example: '信息技术, 医疗' },
  { type: 'position', label: '岗位', category: 'text', subCategory: 'company', recommended: false, example: '产品经理, 架构师' },
  { type: 'job_title', label: '职称', category: 'text', subCategory: 'company', recommended: false, example: '资深工程师, 总监' },
  { type: 'buzzword', label: '公司口号', category: 'text', subCategory: 'company', recommended: false, example: '赋能数字化转型' },
  { type: 'buzzword_middle', label: '热词中缀', category: 'text', subCategory: 'company', recommended: false, example: '创新, 智能' },
  { type: 'buzzword_tail', label: '热词后缀', category: 'text', subCategory: 'company', recommended: false, example: '解决方案, 平台' },
  { type: 'catch_phrase', label: '公司标语', category: 'text', subCategory: 'company', recommended: false, example: 'We lead the market' },
  { type: 'bs_verb', label: 'BS动词', category: 'text', subCategory: 'company', recommended: false, example: 'enable, empower' },
  { type: 'bs_adj', label: 'BS形容词', category: 'text', subCategory: 'company', recommended: false, example: 'seamless, scalable' },
  { type: 'bs_noun', label: 'BS名词', category: 'text', subCategory: 'company', recommended: false, example: 'solutions, platforms' },
  { type: 'bs', label: 'BS短语', category: 'text', subCategory: 'company', recommended: false, example: 'seamless scalable platforms' },

  { type: 'ipv4', label: 'IPv4 地址', category: 'text', subCategory: 'internet', recommended: false, example: '192.168.1.1' },
  { type: 'ipv6', label: 'IPv6 地址', category: 'text', subCategory: 'internet', recommended: false, example: '2001:db8::1' },
  { type: 'ip_address', label: 'IP 地址', category: 'text', subCategory: 'internet', recommended: false, example: '192.168.1.1' },
  { type: 'ip', label: 'IP 通用', category: 'text', subCategory: 'internet', recommended: false, example: '192.168.1.1 / 2001:db8::1' },
  { type: 'mac_address', label: 'MAC 地址', category: 'text', subCategory: 'internet', recommended: false, example: '00:1A:2B:3C:4D:5E' },
  { type: 'url', label: 'URL', category: 'text', subCategory: 'internet', recommended: false, example: 'www.example.com/product' },
  { type: 'user_agent', label: 'User Agent', category: 'text', subCategory: 'internet', recommended: false, example: 'Mozilla/5.0...' },

  { type: 'country', label: '国家(通用)', category: 'text', subCategory: 'address', recommended: false, example: '中国, 美国, 日本' },
  { type: 'country_name', label: '国家名', category: 'text', subCategory: 'address', recommended: false, example: '中国, 美国, 日本' },
  { type: 'country_code', label: '国家代码', category: 'text', subCategory: 'address', recommended: false, example: 'CN, US, JP' },
  { type: 'city', label: '城市', category: 'text', subCategory: 'address', recommended: false, example: '北京, 上海, 深圳' },
  { type: 'city_prefix', label: '城市前缀', category: 'text', subCategory: 'address', recommended: false, example: 'New, Old, North' },
  { type: 'city_suffix', label: '城市后缀', category: 'text', subCategory: 'address', recommended: false, example: 'ville, town, burgh' },
  { type: 'state_name', label: '州/省', category: 'text', subCategory: 'address', recommended: false, example: 'California, 广东' },
  { type: 'state_abbr', label: '州缩写', category: 'text', subCategory: 'address', recommended: false, example: 'CA, NY, TX' },
  { type: 'street_name', label: '街道名', category: 'text', subCategory: 'address', recommended: false, example: '建国路, 长安街' },
  { type: 'street_suffix', label: '街道后缀', category: 'text', subCategory: 'address', recommended: false, example: 'Street, Avenue' },
  { type: 'zip_code', label: '邮编', category: 'text', subCategory: 'address', recommended: false, example: '100010' },
  { type: 'post_code', label: '邮编(通用)', category: 'text', subCategory: 'address', recommended: false, example: '90210' },
  { type: 'building_number', label: '楼号', category: 'text', subCategory: 'address', recommended: false, example: '100, 200, 300' },
  { type: 'secondary_address', label: '二级地址', category: 'text', subCategory: 'address', recommended: false, example: 'Apt. 3B' },
  { type: 'secondary_address_type', label: '二级地址类型', category: 'text', subCategory: 'address', recommended: false, example: 'Apt., Suite' },
  { type: 'latitude', label: '纬度', category: 'text', subCategory: 'address', recommended: false, example: '39.9042' },
  { type: 'longitude', label: '经度', category: 'text', subCategory: 'address', recommended: false, example: '116.4074' },
  { type: 'geohash', label: 'GeoHash', category: 'text', subCategory: 'address', recommended: false, example: 's33u1Kxp' },
  { type: 'timezone', label: '时区', category: 'text', subCategory: 'address', recommended: false, example: 'Asia/Shanghai' },

  { type: 'sentence', label: '句子', category: 'text', subCategory: 'content', recommended: false, example: '这是一个测试句子。' },
  { type: 'sentences', label: '多句', category: 'text', subCategory: 'content', recommended: false, example: '句1。句2。句3。' },
  { type: 'words', label: '单词序列', category: 'text', subCategory: 'content', recommended: false, example: '创新驱动未来发展' },
  { type: 'word', label: '单个单词', category: 'text', subCategory: 'content', recommended: false, example: 'innovation' },
  { type: 'paragraph', label: '段落', category: 'text', subCategory: 'content', recommended: false, example: '多句组成的段落...' },
  { type: 'paragraphs', label: '多段落', category: 'text', subCategory: 'content', recommended: false, example: '段落1\n\n段落2' },
  { type: 'md_italic', label: 'MD 斜体', category: 'text', subCategory: 'content', recommended: false, example: '*italic text*' },
  { type: 'md_bold', label: 'MD 粗体', category: 'text', subCategory: 'content', recommended: false, example: '**bold text**' },
  { type: 'md_link', label: 'MD 链接', category: 'text', subCategory: 'content', recommended: false, example: '[link](https://...)' },
  { type: 'md_bullet', label: 'MD 列表项', category: 'text', subCategory: 'content', recommended: false, example: '- item' },
  { type: 'md_list', label: 'MD 编号列表', category: 'text', subCategory: 'content', recommended: false, example: '1. first\n2. second' },
  { type: 'md_blockquote_single', label: 'MD 引用(单)', category: 'text', subCategory: 'content', recommended: false, example: '> quote' },
  { type: 'md_blockquote_multi', label: 'MD 引用(多)', category: 'text', subCategory: 'content', recommended: false, example: '> line1\n> line2' },
  { type: 'md_code', label: 'MD 代码', category: 'text', subCategory: 'content', recommended: false, example: '`code`' },

  { type: 'currency_code', label: '币种代码', category: 'text', subCategory: 'finance', recommended: false, example: 'CNY, USD, EUR' },
  { type: 'currency_name', label: '币种名', category: 'text', subCategory: 'finance', recommended: false, example: '人民币, 美元' },
  { type: 'currency_symbol', label: '币种符号', category: 'text', subCategory: 'finance', recommended: false, example: '¥, $, €' },
  { type: 'bic', label: 'BIC 代码', category: 'text', subCategory: 'finance', recommended: false, example: 'BKCHCNBJ' },
  { type: 'isin', label: 'ISIN 代码', category: 'text', subCategory: 'finance', recommended: false, example: 'CNE1000002V2' },
  { type: 'credit_card_number', label: '信用卡号', category: 'text', subCategory: 'finance', recommended: false, example: '4532-1234-5678-9012' },

  { type: 'uuid_v1', label: 'UUID v1', category: 'text', subCategory: 'tech', recommended: false, example: 'a1b2c3d0-e4f5-...' },
  { type: 'uuid_v3', label: 'UUID v3', category: 'text', subCategory: 'tech', recommended: false, example: 'a1b2c3d0-e4f5-...' },
  { type: 'uuid_v4', label: 'UUID v4', category: 'text', subCategory: 'tech', recommended: false, example: '550e8400-e29b-...' },
  { type: 'uuid_v5', label: 'UUID v5', category: 'text', subCategory: 'tech', recommended: false, example: 'a1b2c3d0-e4f5-...' },
  { type: 'ferroid_ulid', label: 'ULID', category: 'text', subCategory: 'tech', recommended: false, example: '01BSJ...' },
  { type: 'ferroid_twitter_id', label: 'Twitter ID', category: 'text', subCategory: 'tech', recommended: false, example: '1234567890' },
  { type: 'ferroid_instagram_id', label: 'Instagram ID', category: 'text', subCategory: 'tech', recommended: false, example: '123456789' },
  { type: 'ferroid_mastodon_id', label: 'Mastodon ID', category: 'text', subCategory: 'tech', recommended: false, example: '10987654...' },
  { type: 'ferroid_discord_id', label: 'Discord ID', category: 'text', subCategory: 'tech', recommended: false, example: '9876543...' },
  { type: 'mime_type', label: 'MIME 类型', category: 'text', subCategory: 'tech', recommended: false, example: 'application/json' },
  { type: 'semver', label: 'SemVer', category: 'text', subCategory: 'tech', recommended: false, example: '2.1.0' },
  { type: 'semver_stable', label: 'SemVer(稳定)', category: 'text', subCategory: 'tech', recommended: false, example: '1.0.0' },
  { type: 'semver_unstable', label: 'SemVer(预发)', category: 'text', subCategory: 'tech', recommended: false, example: '0.1.0-alpha' },
  { type: 'file_path', label: '文件路径', category: 'text', subCategory: 'tech', recommended: false, example: '/home/user/data.csv' },
  { type: 'file_name', label: '文件名', category: 'text', subCategory: 'tech', recommended: false, example: 'report_2025.pdf' },
  { type: 'file_extension', label: '文件扩展', category: 'text', subCategory: 'tech', recommended: false, example: 'pdf, csv, json' },
  { type: 'dir_path', label: '目录路径', category: 'text', subCategory: 'tech', recommended: false, example: '/var/data/' },

  { type: 'image_url', label: '图片 URL', category: 'text', subCategory: 'media', recommended: false, example: 'https://picsum.photos/...' },
  { type: 'image_url_with_seed', label: '图片(种子)', category: 'text', subCategory: 'media', recommended: false, example: 'picsum.photos/seed/abc...' },
  { type: 'image_url_grayscale', label: '图片(灰度)', category: 'text', subCategory: 'media', recommended: false, example: 'picsum.photos/...?grayscale' },
  { type: 'image_url_blur', label: '图片(模糊)', category: 'text', subCategory: 'media', recommended: false, example: 'picsum.photos/...?blur=2' },
  { type: 'image_url_custom', label: '图片(自定义)', category: 'text', subCategory: 'media', recommended: false, example: 'picsum.photos/id/237/...' },
  { type: 'hex_color', label: 'Hex 颜色', category: 'text', subCategory: 'media', recommended: false, example: '#3a8fd4' },
  { type: 'rgb_color', label: 'RGB 颜色', category: 'text', subCategory: 'media', recommended: false, example: 'rgb(58,143,212)' },
  { type: 'rgba_color', label: 'RGBA 颜色', category: 'text', subCategory: 'media', recommended: false, example: 'rgba(58,143,212,0.8)' },
  { type: 'hsl_color', label: 'HSL 颜色', category: 'text', subCategory: 'media', recommended: false, example: 'hsl(216,78%,52%)' },
  { type: 'hsla_color', label: 'HSLA 颜色', category: 'text', subCategory: 'media', recommended: false, example: 'hsla(216,78%,52%,0.8)' },
  { type: 'color', label: '颜色(通用)', category: 'text', subCategory: 'media', recommended: false, example: '#3a8fd4' },

  { type: 'regex', label: '自定义正则', category: 'text', subCategory: 'other', recommended: false, example: '[A-Z]{3}-[0-9]{4}' },
  { type: 'template', label: '模板字符串', category: 'text', subCategory: 'other', recommended: false, example: '{{name}}_{{int:1-99}}' },
  { type: 'constant', label: '常量', category: 'text', subCategory: 'other', recommended: false, example: '固定值' },
  { type: 'isbn', label: 'ISBN', category: 'text', subCategory: 'other', recommended: false, example: '978-0-306-40615-7' },
  { type: 'isbn10', label: 'ISBN-10', category: 'text', subCategory: 'other', recommended: false, example: '0-306-40615-2' },
  { type: 'isbn13', label: 'ISBN-13', category: 'text', subCategory: 'other', recommended: false, example: '978-0-306-40615-7' },
  { type: 'rfc_status', label: 'RFC 状态码', category: 'text', subCategory: 'other', recommended: false, example: '200, 301, 404' },
  { type: 'valid_status', label: 'HTTP 有效码', category: 'text', subCategory: 'other', recommended: false, example: '200, 201, 204' },
  { type: 'licence_plate', label: '车牌号', category: 'text', subCategory: 'other', recommended: false, example: 'ABC-1234' },
  { type: 'health_insurance', label: '医保号', category: 'text', subCategory: 'other', recommended: false, example: '123456789012' },
  { type: 'sequence', label: '序列循环', category: 'text', subCategory: 'other', recommended: false, example: 'A → B → C → A...' },
  { type: 'weighted', label: '加权随机', category: 'text', subCategory: 'other', recommended: false, example: '70%A, 30%B' },

  { type: 'boolean', label: '均匀布尔 50/50', category: 'boolean', recommended: true, example: 'True, False, True...' },

  { type: 'foreign_key', label: '外键约束', category: 'foreign_key', recommended: false, example: '引用目标表主键' },
]

const GENERATOR_PARAM_SCHEMA: Partial<Record<GeneratorType, ParamFieldDef[]>> = {
  auto_increment: [
    { name: 'start', label: '起始值', type: 'number', default: 1 },
    { name: 'step', label: '步长', type: 'number', default: 1, min: 1 },
  ],
  random_int: [
    { name: 'min', label: '最小值', type: 'number', default: 0 },
    { name: 'max', label: '最大值', type: 'number', default: 100 },
  ],
  normal: [
    { name: 'mean', label: '均值 μ', type: 'number', default: 500 },
    { name: 'stdDev', label: '标准差 σ', type: 'number', default: 200, min: 0 },
  ],
  log_normal: [
    { name: 'median', label: '中位数', type: 'number', default: 500, min: 0 },
    { name: 'dispersion', label: '分散度', type: 'number', default: 0.8, min: 0 },
  ],
  random_walk: [
    { name: 'start', label: '起始值', type: 'number', default: 1000 },
    { name: 'step', label: '步长', type: 'number', default: 10 },
    { name: 'volatility', label: '波动', type: 'number', default: 5, min: 0 },
  ],
  random_float: [
    { name: 'min', label: '最小值', type: 'number', default: 0 },
    { name: 'max', label: '最大值', type: 'number', default: 1 },
    { name: 'precision', label: '小数位', type: 'number', default: 2, min: 0, max: 10 },
  ],
  random_decimal: [
    { name: 'min', label: '最小值', type: 'number', default: 0 },
    { name: 'max', label: '最大值', type: 'number', default: 1000 },
    { name: 'scale', label: '小数位', type: 'number', default: 2, min: 0, max: 10 },
  ],
  number_with_format: [
    { name: 'fmt', label: '格式模板', type: 'text', default: '###-###-####' },
  ],
  boolean: [
    { name: 'ratio', label: 'True 比例 (%)', type: 'number', default: 50, min: 0, max: 100 },
  ],
  datetime: [
    { name: 'min', label: '起始日期', type: 'text', default: '2020-01-01' },
    { name: 'max', label: '结束日期', type: 'text', default: '2026-12-31' },
  ],
  datetime_before: [
    { name: 'before', label: '早于', type: 'text', default: '2026-12-31' },
  ],
  datetime_after: [
    { name: 'after', label: '晚于', type: 'text', default: '2020-01-01' },
  ],
  datetime_between: [
    { name: 'start', label: '开始', type: 'text', default: '2020-01-01' },
    { name: 'end', label: '结束', type: 'text', default: '2026-12-31' },
  ],
  sequential_date: [
    { name: 'start', label: '起始', type: 'text', default: '2025-01-01 00:00:00' },
    { name: 'stepSeconds', label: '步长(秒)', type: 'number', default: 3600, min: 1 },
  ],
  sequential_date_with_gaps: [
    { name: 'start', label: '起始', type: 'text', default: '2025-01-01' },
    { name: 'stepSeconds', label: '步长(秒)', type: 'number', default: 86400, min: 1 },
    { name: 'missProbability', label: '缺失概率', type: 'number', default: 0.05, min: 0, max: 1 },
  ],
  date: [
    { name: 'min', label: '起始', type: 'text', default: '2020-01-01' },
    { name: 'max', label: '结束', type: 'text', default: '2026-12-31' },
  ],
  password: [
    { name: 'min', label: '最小长度', type: 'number', default: 8, min: 1 },
    { name: 'max', label: '最大长度', type: 'number', default: 16, min: 1 },
  ],
  words: [
    { name: 'min', label: '最少词数', type: 'number', default: 3, min: 1 },
    { name: 'max', label: '最多词数', type: 'number', default: 8, min: 1 },
  ],
  sentence: [
    { name: 'min', label: '最少词数', type: 'number', default: 3, min: 1 },
    { name: 'max', label: '最多词数', type: 'number', default: 12, min: 1 },
  ],
  sentences: [
    { name: 'min', label: '最少句数', type: 'number', default: 2, min: 1 },
    { name: 'max', label: '最多句数', type: 'number', default: 5, min: 1 },
  ],
  paragraph: [
    { name: 'count', label: '句子数', type: 'number', default: 4, min: 1 },
  ],
  paragraphs: [
    { name: 'count', label: '段落数', type: 'number', default: 2, min: 1 },
  ],
  constant: [
    { name: 'value', label: '常量值', type: 'text', default: '' },
  ],
  regex: [
    { name: 'pattern', label: '正则表达式', type: 'text', default: '[a-z]{5,10}' },
  ],
  template: [
    { name: 'template', label: '模板字符串', type: 'text', default: '' },
  ],
  sequence: [
    { name: 'values', label: '值列表(逗号分隔)', type: 'text', default: '' },
    { name: 'cycle', label: '循环', type: 'boolean', default: true },
  ],
  weighted: [
    { name: 'choices', label: '权重配置(值:权重,...)', type: 'text', default: '' },
  ],
}

const GENERATOR_MODULES: Partial<Record<GeneratorType, string>> = {
  auto_increment: '自定义(numeric)',
  random_int: '自定义(numeric)',
  normal: '自定义(numeric/Box-Muller)',
  log_normal: '自定义(numeric/Box-Muller)',
  random_walk: '自定义(numeric/Wiener)',
  random_float: '自定义(numeric)',
  random_decimal: '自定义(numeric)',
  digit: 'number',
  number_with_format: 'number',
  datetime: 'chrono',
  datetime_between: 'chrono',
  datetime_before: 'chrono',
  datetime_after: 'chrono',
  sequential_date: '自定义(chrono)',
  sequential_date_with_gaps: '自定义(chrono)',
  date: 'chrono',
  time: 'chrono',
  duration: 'chrono',
  name: 'name',
  name_with_title: 'name',
  first_name: 'name',
  last_name: 'name',
  title: 'name',
  suffix: 'name',
  phone_number: 'phone_number',
  cell_number: 'phone_number',
  email: 'internet',
  safe_email: 'internet',
  free_email: 'internet',
  free_email_provider: 'internet',
  domain_suffix: 'internet',
  username: 'internet',
  password: 'internet',
  company_name: 'company',
  company_suffix: 'company',
  industry: 'company',
  profession: 'company',
  seniority: 'company',
  field: 'job',
  position: 'job',
  job_title: 'job',
  buzzword: 'company',
  buzzword_middle: 'company',
  buzzword_tail: 'company',
  catch_phrase: 'company',
  bs_verb: 'company',
  bs_adj: 'company',
  bs_noun: 'company',
  bs: 'company',
  currency_code: 'currency',
  currency_name: 'currency',
  currency_symbol: 'currency',
  bic: 'finance',
  isin: 'finance',
  credit_card_number: 'creditcard',
  ipv4: 'internet',
  ipv6: 'internet',
  ip_address: 'internet',
  ip: 'internet',
  mac_address: 'internet',
  url: 'internet',
  user_agent: 'internet',
  mime_type: 'filesystem',
  semver: 'filesystem',
  semver_stable: 'filesystem',
  semver_unstable: 'filesystem',
  file_path: 'filesystem',
  file_name: 'filesystem',
  file_extension: 'filesystem',
  dir_path: 'filesystem',
  country: 'address',
  country_name: 'address',
  country_code: 'address',
  city: 'address',
  city_prefix: 'address',
  city_suffix: 'address',
  state_name: 'address',
  state_abbr: 'address',
  street_name: 'address',
  street_suffix: 'address',
  zip_code: 'address',
  post_code: 'address',
  building_number: 'address',
  secondary_address: 'address',
  secondary_address_type: 'address',
  latitude: 'address',
  longitude: 'address',
  geohash: 'address',
  timezone: 'address',
  image_url: 'picsum',
  image_url_with_seed: 'picsum',
  image_url_grayscale: 'picsum',
  image_url_blur: 'picsum',
  image_url_custom: 'picsum',
  hex_color: 'color',
  rgb_color: 'color',
  rgba_color: 'color',
  hsl_color: 'color',
  hsla_color: 'color',
  color: 'color',
  uuid_v1: 'uuid',
  uuid_v3: 'uuid',
  uuid_v4: 'uuid',
  uuid_v5: 'uuid',
  ferroid_ulid: 'ferroid',
  ferroid_twitter_id: 'ferroid',
  ferroid_instagram_id: 'ferroid',
  ferroid_mastodon_id: 'ferroid',
  ferroid_discord_id: 'ferroid',
  sentence: 'lorem',
  sentences: 'lorem',
  words: 'lorem',
  word: 'lorem',
  paragraph: 'lorem',
  paragraphs: 'lorem',
  md_italic: 'markdown',
  md_bold: 'markdown',
  md_link: 'markdown',
  md_bullet: 'markdown',
  md_list: 'markdown',
  md_blockquote_single: 'markdown',
  md_blockquote_multi: 'markdown',
  md_code: 'markdown',
  isbn: 'barcode',
  isbn10: 'barcode',
  isbn13: 'barcode',
  rfc_status: 'http',
  valid_status: 'http',
  licence_plate: 'automotive',
  health_insurance: 'administrative',
  boolean: 'boolean',
  regex: '自定义(regex)',
  template: '自定义(template)',
  constant: '自定义(constant)',
  foreign_key: '自定义(fk)',
  sequence: '自定义(sequence)',
  weighted: '自定义(weighted)',
}

const DATA_TYPE_OPTIONS: Array<{ label: string; value: string }> = [
  { label: 'INTEGER', value: 'integer' },
  { label: 'BIGINT', value: 'bigint' },
  { label: 'FLOAT', value: 'float' },
  { label: 'DOUBLE', value: 'double' },
  { label: 'DECIMAL', value: 'decimal' },
  { label: 'VARCHAR', value: 'varchar' },
  { label: 'TEXT', value: 'text' },
  { label: 'BOOLEAN', value: 'boolean' },
  { label: 'DATE', value: 'date' },
  { label: 'DATETIME', value: 'datetime' },
  { label: 'TIMESTAMP', value: 'timestamp' },
  { label: 'UUID', value: 'uuid' },
  { label: 'BLOB', value: 'blob' },
]

interface FilterTab {
  key: string
  label: string
}

const FILTER_TABS: FilterTab[] = [
  { key: 'all', label: '全部' },
  { key: 'numeric', label: '📊 数字型' },
  { key: 'date', label: '📅 日期型' },
  { key: 'text', label: '📝 文本型' },
  { key: 'boolean', label: '✅ 布尔型' },
  { key: 'foreign_key', label: '🔗 外键' },
]

// ===================== 状态 =====================

const props = defineProps<{
  show: boolean
  generatorType: GeneratorType
  currentParams: Record<string, unknown>
  columnName: string
  columnIndex: number
  columnDataType: ColumnDataType
  columnNullableRatio: number
  columnUnique: boolean
  allColumns: Array<{ name: string; dataType: string }>
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
  apply: [index: number, type: GeneratorType, params: Record<string, unknown>, fieldName: string, dataType: ColumnDataType, nullableRatio: number, unique: boolean]
}>()

const visible = ref(props.show)
const activeFilter = ref('all')
const localGeneratorType = ref<GeneratorType>(props.generatorType)
const localParams = ref<Record<string, unknown>>({ ...props.currentParams })
const localFieldName = ref(props.columnName)
const localDataType = ref<ColumnDataType>(props.columnDataType)
const localNullRatio = ref(props.columnNullableRatio)
const localUnique = ref(props.columnUnique)
const tsDateColumn = ref<string | null>(null)
const tsTrend = ref<'growth' | 'decline' | 'cycle' | 'walk'>('growth')
const tsCycleLength = ref(30)
const tsVolatility = ref(10)

watch(() => props.show, (val) => {
  visible.value = val
  if (val) {
    localGeneratorType.value = props.generatorType
    localParams.value = { ...props.currentParams }
    localFieldName.value = props.columnName
    localDataType.value = props.columnDataType
    localNullRatio.value = props.columnNullableRatio
    localUnique.value = props.columnUnique
    const genDesc = GENERATOR_LIST.find(g => g.type === props.generatorType)
    if (genDesc) {
      activeFilter.value = genDesc.category
    }
  }
})

watch(visible, (val) => emit('update:show', val))

// ===================== 计算属性 =====================

const dataTypeOptions = DATA_TYPE_OPTIONS

const filterTabs = FILTER_TABS

const filteredGenerators = computed(() => {
  if (activeFilter.value === 'all') return GENERATOR_LIST
  return GENERATOR_LIST.filter(g => g.category === activeFilter.value)
})

const paramFields = computed(() =>
  GENERATOR_PARAM_SCHEMA[localGeneratorType.value] ?? []
)

const generatorModule = computed(() =>
  GENERATOR_MODULES[localGeneratorType.value] ?? localGeneratorType.value
)

const dateColumnOptions = computed(() =>
  props.allColumns
    .filter(c => c.name !== props.columnName)
    .map(c => ({ label: c.name, value: c.name }))
)

const trendOptions = [
  { label: '增长', value: 'growth' },
  { label: '下降', value: 'decline' },
  { label: '周期波动', value: 'cycle' },
  { label: '随机游走', value: 'walk' },
]

const showTimeSeries = computed(() => {
  const gen = GENERATOR_LIST.find(g => g.type === localGeneratorType.value)
  return gen?.category === 'numeric'
})

// ===================== 方法 =====================

function selectGenerator(gen: GeneratorDesc) {
  localGeneratorType.value = gen.type
  localParams.value = {}
}

function setParam(name: string, value: unknown) {
  localParams.value = { ...localParams.value, [name]: value }
}

function restoreDefault() {
  localParams.value = {}
  const defaultGen = GENERATOR_LIST.find(g => g.category === activeFilter.value && g.recommended)
  if (defaultGen) {
    localGeneratorType.value = defaultGen.type
  }
}

function onApply() {
  emit('apply', props.columnIndex, localGeneratorType.value, localParams.value, localFieldName.value, localDataType.value, localNullRatio.value, localUnique.value)
  visible.value = false
}

function onClose() {
  visible.value = false
}
</script>

<style scoped>
.drawer-body {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-header {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 4px;
}

.field-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.field-label {
  font-size: 12px;
  color: var(--color-text-muted);
  width: 48px;
  flex-shrink: 0;
}

.section-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  margin-top: 4px;
  margin-bottom: 4px;
}

.filter-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.generator-list {
  max-height: 240px;
  overflow-y: auto;
  border: 1px solid var(--border-color);
  border-radius: 4px;
}

.generator-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 8px;
  cursor: pointer;
  border-bottom: 1px solid var(--border-color-light);
  font-size: 12px;
}

.generator-item:last-child {
  border-bottom: none;
}

.generator-item:hover {
  background: var(--hover-color);
}

.generator-item.selected {
  background: var(--brand-10p);
  border-left: 2px solid var(--brand-accent);
}

.gen-info {
  display: flex;
  gap: 6px;
  align-items: center;
}

.recommend-badge {
  font-size: 10px;
  color: #18a058;
  font-weight: 500;
}

.gen-name {
  font-weight: 500;
}

.gen-type {
  font-size: 10px;
  color: var(--color-text-muted);
  font-family: monospace;
}

.gen-example {
  font-size: 10px;
  color: var(--color-text-muted);
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex-shrink: 0;
}

.empty-list {
  padding: 20px;
  text-align: center;
  font-size: 12px;
  color: var(--color-text-muted);
}

.params-form {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.param-field {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.param-field-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.empty-params {
  text-align: center;
  padding: 8px 0;
  font-size: 13px;
  color: var(--color-text-muted);
}

.timeseries-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.ts-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.ts-label {
  font-size: 12px;
  color: var(--color-text-muted);
  width: 72px;
  flex-shrink: 0;
}

.other-options {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.opt-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.opt-label {
  font-size: 12px;
  color: var(--color-text-muted);
}

.opt-pct {
  font-size: 12px;
  color: var(--brand-accent);
  font-weight: 500;
}

.source-info {
  margin-top: 4px;
  font-size: 11px;
  color: var(--color-text-muted);
  text-align: right;
}

.drawer-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.footer-right {
  display: flex;
  gap: 8px;
}
</style>