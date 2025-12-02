# PicACG Windows 漫画列表页面实现分析

> 源码路径: `C:\Users\ffqi\dev\py\picacg-windows`

## 1. 分类点击后显示漫画列表的流程

**流程链：**
1. 用户在分类列表（`CategoryView`）中点击分类
2. 触发 `SelectItem` 回调
3. 调用 `QtOwner().OpenSearchByCategory(widget.nameLable.text())`
4. 导航到搜索视图并传递分类名称

**关键代码位置：**
- `src/view/category/category_view.py` (第 53-66 行)

```python
def SelectItem(self, item):
    assert isinstance(item, QListWidgetItem)
    widget = self.bookList.itemWidget(item)
    if widget.id == "1":
        QtOwner().OpenRank()
    elif widget.id == "2":
        QtOwner().OpenComment("5822a6e3ad7ede654696e482")
    elif widget.id == "3":
        QtOwner().OpenSearchByText("")
    elif widget.id == "4":
        QtOwner().OpenIndex()
    else:
        QtOwner().OpenSearchByCategory(widget.nameLable.text())
```

---

## 2. 漫画列表 UI 布局（卡片样式）

### 卡片大小和响应式缩放

| 类型 | 基础宽度 | 基础高度 | 缩放比例 |
|------|----------|----------|----------|
| 漫画卡片 | 250px | 340px | `Setting.CoverSize` (%) |
| 分类卡片 | 300px | 300px | `Setting.CategorySize` (%) |

### 卡片结构（ComicItemWidget）

| 组件 | 用途 | 位置 |
|------|------|------|
| `picLabel` | 封面图片显示 | 上部（固定宽高） |
| `toolButton` | 新章节标记按钮 | 左侧小按钮 |
| `categoryLabel` | 分类标签 | 左上角 |
| `starButton` | 点赞计数 | 下方左侧 |
| `timeLabel` | 更新时间 | 下方右侧 |
| `nameLable` | 标题（支持多行） | 下部 |

**关键代码位置：**
- `src/component/widget/comic_item_widget.py` (第 14-88 行)

```python
class ComicItemWidget(QWidget, Ui_ComicItem):
    def __init__(self, isCategory=False, isShiled=False):
        # 设置卡片尺寸（支持DPI缩放）
        if not isCategory:
            rate = Setting.CoverSize.value  # 用户设置的缩放比例（%）
            baseW = 250
            baseH = 340
        else:
            rate = Setting.CategorySize.value
            baseW = 300
            baseH = 300

        width = baseW * rate / 100
        height = baseH * rate / 100
        self.picLabel.setFixedSize(width, height)

        # 标题支持自动换行，最大宽度限制
        self.nameLable.setMaximumWidth(width-20)
        self.nameLable.setWordWrap(True)
```

### 卡片样式特性

- 支持屏蔽显示（灰色遮罩）
- 支持 Waifu2x 图片超分辨率处理
- 支持图片加载失败提示
- 支持图片缓存和异步加载

---

## 3. 列表网格布局

**列表配置（ComicListWidget）：**

| 配置项 | 值 | 说明 |
|--------|-----|------|
| 布局方向 | `LeftToRight` | 从左到右 |
| 自动换行 | `True` | 启用 |
| 调整模式 | `Adjust` | 自动调整 |
| 水平滚动条 | `ScrollBarAlwaysOff` | 隐藏 |

**关键代码位置：**
- `src/component/list/comic_list_widget.py` (第 18-36 行)

```python
class ComicListWidget(BaseListWidget):
    def __init__(self, parent):
        self.resize(800, 600)
        self.setFrameShape(QListView.NoFrame)  # 无边框
        self.setFlow(QListView.LeftToRight)    # 从左到右排列
        self.setWrapping(True)                  # 自动换行
        self.setResizeMode(QListView.Adjust)   # 自动调整卡片大小
        self.setHorizontalScrollBarPolicy(Qt.ScrollBarAlwaysOff)  # 隐藏水平滚动条
```

---

## 4. 分页逻辑

### 分页架构

| 层级 | 组件 | 职责 |
|------|------|------|
| 模型层 | `BaseListWidget` | 管理页码状态 |
| 视图层 | `SearchView` | 控制分页 UI 和数据加载 |
| 网络层 | `req.py` | 构建分页 API 请求 |

### 分页 UI 组件（SearchView）

| 组件 | 用途 |
|------|------|
| `spinBox` | 页码输入框（范围 1 到 pages） |
| `label` | 显示当前页码（"页: 1/10" 格式） |
| `jumpPage` | 跳转按钮 |
| `comboBox` | 排序方式选择 |

**关键代码位置：**
- `src/view/search/search_view.py` (第 199-387 行)

```python
# 分页状态管理
def UpdatePage(self, page, pages):
    self.page = page
    self.pages = pages

# 获取分页显示文字
def GetPageStr(self):
    return Str.GetStr(Str.Page) + ": " + str(self.page) + "/" + str(self.pages)

# 跳转页面
def JumpPage(self):
    page = int(self.spinBox.text())
    if page > self.bookList.pages:
        return
    self.bookList.page = page
    self.bookList.clear()
    if not self.categories:
        self.SendSearch(page)
    else:
        self.SendSearchCategories(page)

# 加载下一页（滚动到底部触发）
def LoadNextPage(self):
    if not self.categories:
        self.SendSearch(self.bookList.page + 1)
    else:
        self.SendSearchCategories(self.bookList.page + 1)
```

### 分页 API 调用

```python
# 分类搜索
class CategoriesSearchReq(ServerReq):
    def __init__(self, page, categories, sort=""):
        categories = quote(categories)
        url = config.Url + "comics?page={}&c={}&s={}".format(page, categories, sort)
        # GET 请求

# 高级搜索
class AdvancedSearchReq(ServerReq):
    def __init__(self, page, categories, keyword="", sort=""):
        url = config.Url + "comics/advanced-search?page={}".format(page)
        # POST: {"categories": categories, "keyword": keyword, "sort": sort}
```

---

## 5. 数据加载流程

```
用户操作
    ↓
SendSearch(page) / SendSearchCategories(page)
    ↓
    ├─ 本地搜索 → SQL查询 → SendLocalBack()
    │                      → AddBookItemByBook()
    │
    └─ 远程搜索 → HTTP请求 → SendSearchBack()
                           → AddBookByDict()
    ↓
更新分页信息：UpdatePage(page, pages)
    ↓
清空列表并渲染 → AddBookItem() / AddBookByDict()
    ↓
异步加载封面图片 → LoadingPicture() → LoadingPictureComplete()
```

### 远程搜索数据加载

```python
def SendSearchBack(self, raw):
    QtOwner().CloseLoading()
    try:
        self.bookList.UpdateState()
        data = json.loads(raw["data"])
        st = raw["st"]
        if st == Status.Ok:
            info = data.get("data").get("comics")
            page = int(info.get("page"))           # 当前页
            pages = int(info.get("pages"))         # 总页数
            self.bookList.UpdatePage(page, pages)  # 更新分页信息
            self.spinBox.setValue(page)
            self.spinBox.setMaximum(pages)
            self.label.setText(self.bookList.GetPageStr())
            for v in info.get("docs", []):         # 遍历漫画数据
                self.bookList.AddBookByDict(v)     # 添加到列表
```

### 漫画项添加方式

| 方法 | 数据源 | 用途 |
|------|--------|------|
| `AddBookByDict()` | API 响应 JSON | 远程搜索、分类、排名 |
| `AddBookItemByBook()` | 本地 Book 对象 | 本地搜索、收藏 |
| `AddBookByLocal()` | LocalData 对象 | 本地阅读 |

```python
# 添加远程搜索结果
def AddBookByDict(self, v):
    _id = v.get("_id")
    title = v.get("title")
    url = v.get("thumb", {}).get("fileServer")
    path = v.get("thumb", {}).get("path")
    likesCount = str(v.get("totalLikes", ""))
    pagesCount = v.get("pagesCount")
    self.AddBookItem(_id, title, categoryStr, url, path, likesCount,
                     "", pagesCount, finished, isShiled=isShiled)
```

---

## 6. 图片异步加载流程

```python
# 异步加载触发（paintEvent）
def paintEvent(self, event):
    if self.url and not self.isLoadPicture and config.IsLoadingPicture:
        self.isLoadPicture = True
        self.PicLoad.emit(self.index)  # 发送加载信号

# 加载处理
def LoadingPicture(self, index):
    widget = self.itemWidget(self.item(index))
    self.AddDownloadTask(widget.url, widget.path,
                        completeCallBack=self.LoadingPictureComplete,
                        backParam=index)

# 完成回调
def LoadingPictureComplete(self, data, status, index):
    if status == Status.Ok:
        widget.SetPicture(data)  # 显示图片
        # 可选：自动应用 Waifu2x 超分
        if Setting.CoverIsOpenWaifu.value:
            self.Waifu2xPicture(indexModel, True)
    else:
        widget.SetPictureErr(status)  # 显示错误
```

---

## 7. 自动加载下一页（无限滚动）

**触发机制（BaseListWidget）：**
- 监听垂直滚动条变化
- 当滚动到底部时自动触发 `LoadNextPage()`

```python
# 滚动监听
def ValueChange(self, v):
    if v >= self.verticalScrollBar().maximum():
        self.ClearWheelEvent()
        self.isLoadingPage = True
        if self.LoadCallBack:
            self.LoadCallBack()  # 触发加载下一页

# 拖拽滚动触发
def OnActionTriggered(self):
    if self.page >= self.pages:
        return
    if self.verticalScrollBar().sliderPosition() == self.verticalScrollBar().maximum():
        self.isLoadingPage = True
        if self.LoadCallBack:
            self.LoadCallBack()
```

---

## 8. 排序选项

### 远程搜索排序

| 显示名称 | API 值 |
|----------|--------|
| 新到旧 | `dd` |
| 旧到新 | `da` |
| 最多爱心 | `ld` |
| 最多绅士指数 | `vd` |

### 本地搜索排序

| 排序字段 | 说明 |
|----------|------|
| 更新时间 | updated_at |
| 创建时间 | created_at |
| 爱心数 | totalLikes |
| 观看数 | totalViews |
| 章节数 | epsCount |
| 图片数 | pagesCount |

---

## 9. 关键文件路径

| 功能 | 文件路径 |
|------|----------|
| 漫画卡片组件 | `src/component/widget/comic_item_widget.py` |
| 漫画列表组件 | `src/component/list/comic_list_widget.py` |
| 基础列表组件 | `src/component/list/base_list_widget.py` |
| 搜索视图 | `src/view/search/search_view.py` |
| 分类视图 | `src/view/category/category_view.py` |
| API 请求 | `src/server/req.py` |
| 卡片 UI 定义 | `src/interface/ui_comic_item.py` |

---

## 10. Rust 移植要点

### 需要实现的组件

1. **ComicCard 组件** - 漫画卡片
   - 固定宽高 (180x300 或可配置)
   - 封面图异步加载
   - 标题（支持多行）
   - 点赞数、更新时间

2. **ComicGrid 布局** - 网格列表
   - FlexWrap 自动换行
   - 响应式列数

3. **分页控制** - 页码导航
   - 当前页/总页数显示
   - 页码跳转输入框
   - 排序选择

4. **数据加载** - API 集成
   - 分类搜索 API
   - 分页参数处理
   - 图片异步加载

5. **滚动加载** - 无限滚动
   - 监听滚动位置
   - 自动加载下一页
