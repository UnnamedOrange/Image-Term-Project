# %%
import numpy as np
import cv2
import queue
from matplotlib import pyplot as plt


# %%
def imshow(img):
    plt.imshow(img, cmap="gray")
    plt.axis("off")
    plt.show()


# %%
def dilate_outside(img):
    """将图像背景膨胀，以消除外面那一圈。返回保留的前景掩码（255 表示保留）。

    Args:
        img: 二维的灰度图像。
    """
    is_visited = np.zeros(img.shape, np.bool_)
    background_mask = np.zeros(img.shape, np.uint8)
    q = queue.Queue()
    q.put((0, 0))
    is_visited[0, 0] = True
    while not q.empty():
        x, y = q.get()
        background_mask[x, y] = 255
        for dx, dy in [(1, 0), (-1, 0), (0, 1), (0, -1)]:
            nx, ny = x + dx, y + dy
            if (
                nx >= 0
                and nx < img.shape[0]
                and ny >= 0
                and ny < img.shape[1]
                and not is_visited[nx, ny]
                and np.abs(int(img[nx, ny]) - int(img[0, 0])) <= 16
            ):
                q.put((nx, ny))
                is_visited[nx, ny] = True
    kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (7, 7), anchor=(3, 3))
    background_mask = cv2.dilate(
        background_mask,
        kernel,
        borderType=cv2.BORDER_CONSTANT,
        borderValue=0,
    )

    foreground_mask = cv2.bitwise_not(background_mask)
    kernel = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (33, 33), anchor=(16, 16))
    foreground_mask = cv2.erode(
        foreground_mask,
        kernel,
        borderType=cv2.BORDER_CONSTANT,
        borderValue=0,
    )

    return foreground_mask


def to_binary(img):
    """将图片按照自定义的规则二值化。

    Args:
        img: YUV 格式的图片。
    """
    ret = np.zeros(img.shape[:2], np.uint8)
    for i in range(img.shape[0]):
        for j in range(img.shape[1]):
            p = img[i, j]
            if p[0] > 160:
                ret[i, j] = 255
    return ret


def remove_block(img, target, len_pred):
    """去除二值化图像中小的连通块，使得背景更加平滑。

    Args:
        img: 二值化的图像。
    """
    is_visited = np.zeros(img.shape, np.bool_)
    mask = np.zeros(img.shape, np.uint8)
    for i in range(img.shape[0]):
        for j in range(img.shape[1]):
            if img[i, j] != target or is_visited[i, j]:
                continue
            q = queue.Queue()
            q.put((i, j))
            is_visited[i, j] = True
            c = []
            while not q.empty():
                x, y = q.get()
                c.append((x, y))
                for dx, dy in [(1, 0), (-1, 0), (0, 1), (0, -1)]:
                    nx, ny = x + dx, y + dy
                    if (
                        nx >= 0
                        and nx < img.shape[0]
                        and ny >= 0
                        and ny < img.shape[1]
                        and not is_visited[nx, ny]
                        and img[nx, ny] == target
                    ):
                        q.put((nx, ny))
                        is_visited[nx, ny] = True

            if len_pred(len(c)):
                for x, y in c:
                    mask[x, y] = 255
    return mask


def generate_pattern(size):
    """生成一个被一定宽度的 0 值圆环包围的模式。

    Args:
        size: 模式的大小。圆环比模式略小一些。
    """
    pattern = np.empty((size, size), np.uint8)
    pattern[:, :] = 255
    cv2.circle(pattern, (size // 2, size // 2), size // 2, 0, 2)
    return pattern


def pattern_match(img, pattern):
    """使用均方差进行模式匹配，返回滑动窗口模式匹配结果。

    Args:
        img: 原图像。
        pattern: 模式图像。

    Returns:
        滑动窗口模式匹配结果。
    """

    ret = np.empty_like(img, np.float32)
    ret.fill(np.inf)
    for y in range(0, img.shape[0] - pattern.shape[0] + 1):
        for x in range(0, img.shape[1] - pattern.shape[1] + 1):
            window = img[y : y + pattern.shape[0], x : x + pattern.shape[1]]
            mse = np.mean((window - pattern) ** 2)
            ret[y + pattern.shape[0] // 2, x + pattern.shape[1] // 2] = mse

    # Fill the edge for a better appearance.
    ret[ret == np.inf] = np.max(
        ret[pattern.shape[0] : -pattern.shape[0], pattern.shape[1] : -pattern.shape[1]]
    )

    return ret


# %%
def main():
    img = cv2.imread("cell.png")
    img = cv2.cvtColor(img, cv2.COLOR_BGR2YUV)
    plt.imsave("gray.png", img[:, :, 0], cmap="gray")
    plt.imsave("u.png", img[:, :, 1], cmap="gray")
    plt.imsave("v.png", img[:, :, 2], cmap="gray")
    imshow(img[:, :, 0])

    mask = dilate_outside(img[:, :, 0])
    img[mask == 0, :] = [0, 128, 128]
    plt.imsave("dilated_outside.png", cv2.cvtColor(img, cv2.COLOR_YUV2RGB))
    imshow(img[:, :, 0])

    img = to_binary(img)
    plt.imsave("binary.png", img, cmap="gray")
    imshow(img)

    pattern = generate_pattern(15)
    plt.imsave("pattern.png", pattern, cmap="gray")
    imshow(pattern)

    response = pattern_match(img, pattern)
    plt.imsave("response.png", response, cmap="gray")
    imshow(response)

    THRESHOLD = 0.33
    matched = (response < THRESHOLD) * 255
    plt.imsave("matched.png", matched, cmap="gray")
    imshow(matched)

    pattern_small = generate_pattern(9)
    plt.imsave("pattern_small.png", pattern_small, cmap="gray")
    imshow(pattern_small)

    response_small = pattern_match(img, pattern_small)
    plt.imsave("response_small.png", response_small, cmap="gray")
    imshow(response_small)

    THRESHOLD = 0.225
    matched_small = (response_small < THRESHOLD) * 255
    plt.imsave("matched_small.png", matched_small, cmap="gray")
    imshow(matched_small)

    matched_all = matched + matched_small
    plt.imsave("matched_all.png", matched_all, cmap="gray")
    imshow(matched_all)


# %%
if __name__ == "__main__":
    main()
